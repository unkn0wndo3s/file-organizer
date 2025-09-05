package fr.unkn0wndo3s.app;

import java.nio.file.Path;
import java.util.List;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

import fr.unkn0wndo3s.core.model.FileRecord;
import fr.unkn0wndo3s.core.planner.FilePlanner;
import fr.unkn0wndo3s.core.planner.PlannerConfig;
import fr.unkn0wndo3s.core.rules.RuleSet;
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

    @Override
    public void start(Stage primaryStage) {
        Platform.setImplicitExit(false);

        // Fenêtre de recherche
        searchWindow = new SearchWindow();

        // Hotkey global
        hotkey = new WindowsHotkeyService(searchWindow::toggle);
        hotkey.start();

        // Tray icon
        TrayUtil.installTray(searchWindow, () -> {
            if (hotkey != null) hotkey.stop();
            if (ioPool != null) ioPool.shutdownNow();
        });

        // Thread IO (scan async)
        ioPool = Executors.newSingleThreadExecutor(r -> {
            Thread t = new Thread(r, "ScanIO");
            t.setDaemon(true);
            return t;
        });

        // ---- SCAN ASYNC (shallow) ----
        ioPool.submit(() -> {
            try {
                var scanner = new FileScanner();

                Path home = Path.of(System.getProperty("user.home"));
                Path downloads = home.resolve("Downloads");
                Path desktop   = home.resolve("Desktop");

                var cfg     = PlannerConfig.windowsDefaults(home);
                var planner = new FilePlanner(new RuleSet(), cfg);

                System.out.println("[scan-shallow] start");
                final int[] count = {0};

                scanner.scanTopLevelStream(List.of(downloads, desktop), entry -> {
                    if (entry.isDirectory()) {
                        if (count[0] < 25) {
                            System.out.printf(" [DIR ] %s%n", entry.path());
                        }
                    } else {
                        var rec = new FileRecord(
                                entry.path(), entry.name(), entry.extensionLower(),
                                entry.sizeBytes(), entry.lastModifiedEpochMillis()
                        );
                        var plan = planner.plan(rec);
                        if (count[0] < 25) {
                            System.out.printf(" [FILE] %s -> %s%n",
                                    plan.record().path(), plan.proposedDestinationPath());
                        }
                    }
                    count[0]++;
                });

                System.out.println("[scan-shallow] done. entries=" + count[0]);
            } catch (Exception e) {
                e.printStackTrace();
            }
        });
        // ---- /SCAN ASYNC ----

        Runtime.getRuntime().addShutdownHook(new Thread(() -> {
            try { if (hotkey != null) hotkey.stop(); } catch (Throwable ignored) {}
            try { if (ioPool != null) ioPool.shutdownNow(); } catch (Throwable ignored) {}
        }));
    }

    public static void main(String[] args) {
        if (!System.getProperty("os.name", "").toLowerCase().contains("win")) {
            System.err.println("Attention: implémentation hotkey Windows active. (Linux: à venir)");
        }
        launch(args);
    }
}
