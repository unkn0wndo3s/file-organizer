package fr.unkn0wndo3s.app;

import fr.unkn0wndo3s.ui.SearchWindow;
import fr.unkn0wndo3s.windows.WindowsHotkeyService;
import javafx.application.Application;
import javafx.application.Platform;
import javafx.stage.Stage;

public class Main extends Application {

    private SearchWindow searchWindow;
    private WindowsHotkeyService hotkey;

    @Override
    public void start(Stage primaryStage) {
        // L’app ne se termine pas quand il n’y a plus de fenêtre visible
        Platform.setImplicitExit(false);

        // Fenêtre de recherche (secondaire, pas d’icône taskbar)
        searchWindow = new SearchWindow();

        // Hotkey global (Ctrl+Espace)
        hotkey = new WindowsHotkeyService(searchWindow::toggle);
        hotkey.start();

        // Ajoute l’icône de tray
        TrayUtil.installTray(searchWindow, () -> {
            if (hotkey != null) hotkey.stop();
        });

        // Ne JAMAIS montrer le primaryStage => aucune icône dans la taskbar
        // primaryStage.show();  // <-- on ne l’appelle pas.
    }

    public static void main(String[] args) {
        if (!System.getProperty("os.name", "").toLowerCase().contains("win")) {
            System.err.println("Hotkey global: implémentation Windows active. (Linux: à venir)");
        }
        launch(args);
    }
}
