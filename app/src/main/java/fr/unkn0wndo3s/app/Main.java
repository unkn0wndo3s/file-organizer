package fr.unkn0wndo3s.app;

import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import fr.unkn0wndo3s.core.scanner.FileScanner;
import fr.unkn0wndo3s.ui.SearchWindow;
import fr.unkn0wndo3s.windows.WindowsHotkeyService;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.stage.Stage;

public class Main extends Application {

    private SearchWindow searchWindow;
    private WindowsHotkeyService hotkey;
    private ExecutorService ioPool;
    private LogWindow logWindow;

    @Override
    public void start(Stage primaryStage) {
        // Pas d’icône barre des tâches
        Platform.setImplicitExit(false);

        // UI : barre + console
        searchWindow = new SearchWindow();
        logWindow = new LogWindow();
        LogBus.addListener(line -> logWindow.append(line));

        // Log les requêtes quand on presse Entrée dans la barre
        searchWindow.setOnSearchSubmit(q -> LogBus.log("[Search] " + q));

        // Hotkey global
        hotkey = new WindowsHotkeyService(searchWindow::toggle);
        hotkey.start();

        // Tray avec item "Console"
        TrayUtil.installTray(searchWindow,
                () -> { // onQuit
                    if (hotkey != null) hotkey.stop();
                    if (ioPool != null) ioPool.shutdownNow();
                },
                () -> logWindow.toggle() // onToggleConsole
        );

        // Thread IO (daemon)
        ioPool = Executors.newSingleThreadExecutor(r -> {
            Thread t = new Thread(r, "ScanIO");
            t.setDaemon(true);
            return t;
        });

        // ---- SCAN ASYNC (Downloads, top-level only, logs en temps réel) ----
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
                    count[0]++;
                });

                LogBus.log("[scan] done. entries=" + count[0]);
            } catch (Exception e) {
                LogBus.log("[scan] error: " + e.getMessage());
                e.printStackTrace();
            }
        });
        // ---- /SCAN ASYNC ----

        // Shutdown propre
        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            try { if (hotkey != null) hotkey.stop(); } catch (Throwable ignored) {}
            try { if (ioPool != null) ioPool.shutdownNow(); } catch (Throwable ignored) {}
            try { LogBus.removeListener(line -> logWindow.append(line)); } catch (Throwable ignored) {}
        }));
    }

    public static void main(String[] args) {
        if (!System.getProperty("os.name", "").toLowerCase().contains("win")) {
            System.err.println("Hotkey global (Windows) seulement dans cette implémentation.");
        }
        launch(args);
    }
}
