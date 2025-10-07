// app/src/main/java/fr/unkn0wndo3s/app/Main.java
package fr.unkn0wndo3s.app;

import java.awt.Desktop;
import java.io.IOException;
import java.lang.reflect.Field;
import java.nio.file.FileSystems;
import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.StandardWatchEventKinds;
import java.nio.file.WatchEvent;
import java.nio.file.WatchKey;
import java.nio.file.WatchService;
import java.nio.file.attribute.DosFileAttributes;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import fr.unkn0wndo3s.core.LogBus;
import fr.unkn0wndo3s.core.fs.FileMover;
import fr.unkn0wndo3s.core.init.DirectoryInitializer;
import fr.unkn0wndo3s.core.scanner.FileScanner;
import fr.unkn0wndo3s.ui.SearchWindow;
import fr.unkn0wndo3s.windows.QuickAccessPinUtil;
import fr.unkn0wndo3s.windows.WindowsHotkeyService;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.stage.Stage;

public class Main extends Application {

    private SearchWindow searchWindow;
    private WindowsHotkeyService hotkey;
    private ExecutorService ioPool;
    private ExecutorService watchPool;
    private LogWindow logWindow;

    private final SearchIndex index = new SearchIndex();

    private WatchService watchService;
    private final Map<WatchKey, Path> watchRoots = new HashMap<>();
    private List<Path> destRoots = List.of();

    @Override
    public void start(Stage primaryStage) {
        Platform.setImplicitExit(false);

        searchWindow = new SearchWindow();
        searchWindow.setOnQuerySubmit(q -> LogBus.log("[search] " + q));
        searchWindow.setOnActivateItem(name -> {
            Path path = index.resolveExact(name);
            if (path == null) { LogBus.log("[open] not found: " + name); return; }
            openPath(path);
        });
        searchWindow.setOnContextMenuItem(name -> {
            Path path = index.resolveExact(name);
            if (path == null) { LogBus.log("[reveal] not found: " + name); return; }
            revealPath(path);
        });

        logWindow = new LogWindow();
        LogBus.addListener(line -> logWindow.append(line));

        hotkey = new WindowsHotkeyService(searchWindow::toggle, new WindowsHotkeyService.HotkeyRegistrationCallback() {
            @Override
            public void onHotkeyRegistered(String hotkeyName) {
                System.out.println("[HOTKEY-REGISTRATION] Hotkey enregistrée: " + hotkeyName);
                LogBus.log("[hotkey] Hotkey enregistrée: " + hotkeyName);
            }
            
            @Override
            public void onHotkeyRegistrationFailed(String errorMessage) {
                System.out.println("[HOTKEY-REGISTRATION] Échec enregistrement: " + errorMessage);
                LogBus.log("[hotkey:error] Échec enregistrement: " + errorMessage);
            }
        });
        hotkey.start();

        TrayUtil.installTray(searchWindow, () -> {
            if (hotkey != null) hotkey.stop();
            if (ioPool != null) ioPool.shutdownNow();
            if (watchPool != null) watchPool.shutdownNow();
            closeWatcher();
        }, () -> logWindow.toggle());

        ioPool = Executors.newSingleThreadExecutor(r -> { var t = new Thread(r, "ScanIO"); t.setDaemon(true); return t; });
        watchPool = Executors.newSingleThreadExecutor(r -> { var t = new Thread(r, "WatchService"); t.setDaemon(true); return t; });

        List<Path> ensured = new ArrayList<>();
        try {
            Path home = Path.of(System.getProperty("user.home"));
            ensured = DirectoryInitializer.ensureBaseAndFolders(home);
            destRoots = List.copyOf(ensured);
            for (Path p : ensured) {
                boolean ok = QuickAccessPinUtil.pin(p);
                LogBus.log("[pin] " + (ok ? "OK " : "KO ") + p);
            }
        } catch (Exception e) {
            LogBus.log("[init:error] " + e.getMessage());
        }

        searchWindow.setItems(index.names());

        ioPool.submit(() -> {
            try {
                var scanner = new FileScanner();
                Path home = Path.of(System.getProperty("user.home"));
                Path downloads = home.resolve("Downloads");

                LogBus.log("[scan] start " + downloads);
                final int[] count = {0};
                scanner.scanTopLevelStream(List.of(downloads), entry -> {
                    LogBus.log((entry.isDirectory() ? "[folder] " : "[file]   ") + entry.path());
                    LogBus.log(String.valueOf(FileMover.movePath(entry.path())));
                    count[0]++;
                });
                LogBus.log("[scan] moved=" + count[0]);

                reloadAll("initial");
                startWatchers();
            } catch (Exception e) {
                LogBus.log("[scan:error] " + e.getMessage());
            }
        });

        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            try { if (hotkey != null) hotkey.stop(); } catch (Throwable ignored) {}
            try { if (ioPool != null) ioPool.shutdownNow(); } catch (Throwable ignored) {}
            try { if (watchPool != null) watchPool.shutdownNow(); } catch (Throwable ignored) {}
            try { LogBus.removeListener(line -> logWindow.append(line)); } catch (Throwable ignored) {}
            closeWatcher();
        }));
    }

    private void openPath(Path path) {
        try {
            if (Desktop.isDesktopSupported()) {
                Desktop.getDesktop().open(path.toFile());
            } else {
                new ProcessBuilder("explorer.exe", path.toString()).start();
            }
            LogBus.log("[open] " + path);
        } catch (Exception ex) {
            LogBus.log("[open:error] " + ex.getMessage());
        }
    }

    private void revealPath(Path path) {
        try {
            new ProcessBuilder("explorer.exe", "/select,", path.toString()).start();
            LogBus.log("[reveal] " + path);
        } catch (Exception ex) {
            LogBus.log("[reveal:error] " + ex.getMessage());
        }
    }

    private void reloadAll(String reason) {
        LogBus.log("[reload] start (" + reason + ")");
        index.clear();
        Platform.runLater(() -> searchWindow.clearItems());
        rebuildIndexFrom(destRoots);
        Platform.runLater(() -> {
            searchWindow.setItems(index.names());
            searchWindow.refreshKeepingFilter();
        });
        LogBus.log("[reload] done, names=" + index.names().size());
    }

    private void rebuildIndexFrom(List<Path> roots) {
        try {
            for (Path root : roots) {
                if (!Files.isDirectory(root) || !Files.isReadable(root)) continue;
                try (var stream = Files.list(root)) {
                    stream.filter(this::isVisible).forEach(index::add);
                } catch (Exception e) {
                    LogBus.log("[index:skip] " + root + " (" + e.getClass().getSimpleName() + ")");
                }
            }
        } catch (Exception e) {
            LogBus.log("[index:error] " + e.getMessage());
        }
    }

    private boolean isVisible(Path p) {
        try {
            if (!Files.exists(p)) return false;
            if (Files.isHidden(p)) return false;
            String name = p.getFileName() == null ? "" : p.getFileName().toString();
            if (name.startsWith(".")) return false;
            String os = System.getProperty("os.name", "").toLowerCase();
            if (os.contains("win")) {
                var a = Files.readAttributes(p, DosFileAttributes.class, LinkOption.NOFOLLOW_LINKS);
                if (a.isHidden() || a.isSystem()) return false;
            }
            return true;
        } catch (IOException e) {
            return false;
        }
    }

    private void startWatchers() {
        try {
            watchService = FileSystems.getDefault().newWatchService();
            for (Path root : destRoots) {
                if (!Files.isDirectory(root)) continue;

                WatchEvent.Kind<?>[] kinds = new WatchEvent.Kind[]{
                        StandardWatchEventKinds.ENTRY_CREATE,
                        StandardWatchEventKinds.ENTRY_DELETE,
                        StandardWatchEventKinds.ENTRY_MODIFY,
                        StandardWatchEventKinds.OVERFLOW
                };

                WatchKey key;
                WatchEvent.Modifier high = tryHighSensitivity();
                if (high != null) key = root.register(watchService, kinds, high);
                else key = root.register(watchService, kinds);

                watchRoots.put(key, root);
                LogBus.log("[watch] " + root);
            }
        } catch (IOException e) {
            LogBus.log("[watch:init:error] " + e.getMessage());
        }

        watchPool.submit(() -> {
            try {
                while (true) {
                    WatchKey key = watchService.take();
                    Path root = watchRoots.get(key);
                    if (root == null) { key.reset(); continue; }

                    boolean needsReload = false;

                    for (WatchEvent<?> ev : key.pollEvents()) {
                        WatchEvent.Kind<?> kind = ev.kind();
                        if (kind == StandardWatchEventKinds.OVERFLOW) { needsReload = true; continue; }

                        @SuppressWarnings("unchecked")
                        WatchEvent<Path> e = (WatchEvent<Path>) ev;
                        Path child = root.resolve(e.context());

                        if (kind == StandardWatchEventKinds.ENTRY_CREATE) { LogBus.log("[watch:+] " + child); needsReload = true; }
                        else if (kind == StandardWatchEventKinds.ENTRY_DELETE) { LogBus.log("[watch:-] " + child); needsReload = true; }
                        else if (kind == StandardWatchEventKinds.ENTRY_MODIFY) { LogBus.log("[watch:~] " + child); needsReload = true; }
                    }

                    if (needsReload) reloadAll("watch");

                    boolean valid = key.reset();
                    if (!valid) {
                        LogBus.log("[watch:end] " + root);
                        watchRoots.remove(key);
                        if (watchRoots.isEmpty()) break;
                    }
                }
            } catch (InterruptedException ie) {
                Thread.currentThread().interrupt();
            } catch (Throwable t) {
                LogBus.log("[watch:error] " + t.getMessage());
            }
        });
    }

    private WatchEvent.Modifier tryHighSensitivity() {
        try {
            Class<?> c = Class.forName("com.sun.nio.file.SensitivityWatchEventModifier");
            Field f = c.getField("HIGH");
            return (WatchEvent.Modifier) f.get(null);
        } catch (Throwable ignore) {
            return null;
        }
    }

    private void closeWatcher() {
        try { if (watchService != null) watchService.close(); } catch (Throwable ignored) {}
        watchRoots.clear();
    }

    public static void main(String[] args) {
        if (!System.getProperty("os.name", "").toLowerCase().contains("win")) {
            System.err.println("Global hotkey (Windows) only in this implementation.");
        }
        launch(args);
    }
}
