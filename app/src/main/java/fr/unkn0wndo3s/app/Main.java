package fr.unkn0wndo3s.app;

import java.awt.Desktop;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

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
    private LogWindow logWindow;

    private final SearchIndex index = new SearchIndex();

    @Override
    public void start(Stage primaryStage) {
        Platform.setImplicitExit(false);

        searchWindow = new SearchWindow();
        searchWindow.setOnQuerySubmit(q -> LogBus.log("[Search] " + q));
        searchWindow.setOnActivateItem(name -> {
            var p = index.resolve(name);
            if (p == null) {
                LogBus.log("[open] not found: " + name);
                return;
            }
            try {
                if (Desktop.isDesktopSupported()) {
                    Desktop.getDesktop().open(p.toFile());
                    LogBus.log("[open] " + p);
                }
            } catch (Exception ex) {
                LogBus.log("[open] error: " + ex.getMessage());
                ex.printStackTrace();
            }
        });

        logWindow = new LogWindow();
        LogBus.addListener(line -> logWindow.append(line));

        hotkey = new WindowsHotkeyService(searchWindow::toggle);
        hotkey.start();

        TrayUtil.installTray(searchWindow, () -> {
            if (hotkey != null) hotkey.stop();
            if (ioPool != null) ioPool.shutdownNow();
        }, () -> logWindow.toggle());

        ioPool = Executors.newSingleThreadExecutor(r -> {
            Thread t = new Thread(r, "ScanIO");
            t.setDaemon(true);
            return t;
        });

        List<Path> ensured = new ArrayList<>();
        try {
            Path home = Path.of(System.getProperty("user.home"));
            ensured = DirectoryInitializer.ensureBaseAndFolders(home);
            LogBus.log("[init] insured files :");
            ensured.forEach(p -> LogBus.log("  - " + p));

            LogBus.log("[pin] quick access (InvokeVerb) …");
            for (Path p : ensured) {
                boolean ok = QuickAccessPinUtil.pin(p);
                LogBus.log("  [" + (ok ? "OK" : "KO") + "] " + p);
            }
        } catch (Exception e) {
            LogBus.log("[init] error init/pin: " + e.getMessage());
            e.printStackTrace();
        }

        final List<Path> destRoots = List.copyOf(ensured);
        searchWindow.setItems(index.names());

        ioPool.submit(() -> {
            try {
                var scanner = new FileScanner();
                Path home = Path.of(System.getProperty("user.home"));
                Path downloads = home.resolve("Downloads");

                LogBus.log("[scan] start (" + downloads + ")");
                final int[] count = {0};

                scanner.scanTopLevelStream(List.of(downloads), entry -> {
                    if (entry.isDirectory()) {
                        LogBus.log("[Folder] " + entry.path());
                    } else {
                        LogBus.log("[File]   " + entry.path());
                    }
                    LogBus.log(FileMover.movePath(entry.path()));
                    count[0]++;
                });
                LogBus.log("[scan] moved entries=" + count[0]);

                rebuildIndexFrom(destRoots);

                Platform.runLater(() -> {
                    searchWindow.setItems(index.names());
                    searchWindow.refreshKeepingFilter();
                });

                LogBus.log("[index] names=" + index.names().size());
            } catch (Exception e) {
                LogBus.log("[scan] error: " + e.getMessage());
                e.printStackTrace();
            }
        });

        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            try { if (hotkey != null) hotkey.stop(); } catch (Throwable ignored) {}
            try { if (ioPool != null) ioPool.shutdownNow(); } catch (Throwable ignored) {}
            try { LogBus.removeListener(line -> logWindow.append(line)); } catch (Throwable ignored) {}
        }));
    }

    private void rebuildIndexFrom(List<Path> roots) {
        index.clear();
        for (Path root : roots) {
            try {
                if (!Files.isDirectory(root) || !Files.isReadable(root)) continue;
                try (var stream = Files.list(root)) {
                    stream.forEach(index::add);
                }
            } catch (Exception e) {
                LogBus.log("[index] skip: " + root + " (" + e.getClass().getSimpleName() + ")");
            }
        }
    }
    

    public static void main(String[] args) {
        if (!System.getProperty("os.name", "").toLowerCase().contains("win")) {
            System.err.println("Global hotkey (Windows) only in this implementation.");
        }
        launch(args);
    }
}
