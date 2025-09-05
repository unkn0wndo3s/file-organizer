package fr.unkn0wndo3s.ui;

import java.util.function.Consumer;

import javafx.application.Platform;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.ListView;
import javafx.scene.control.TextField;
import javafx.scene.input.KeyCode;
import javafx.scene.layout.VBox;
import javafx.stage.Modality;
import javafx.stage.Stage;
import javafx.stage.StageStyle;

public class SearchWindow {
    private final Stage stage;
    private final TextField input;
    private final ListView<String> results;

    private Consumer<String> onSearchSubmit;

    public SearchWindow() {
        stage = new Stage(StageStyle.UNDECORATED);
        stage.initModality(Modality.NONE);
        stage.setAlwaysOnTop(true);

        input = new TextField();
        input.setPromptText("Tape pour chercher…");
        input.getStyleClass().add("search-input"); // classe CSS spécifique
        input.setOnKeyPressed(e -> {
            if (e.getCode() == KeyCode.ESCAPE) hide();
            if (e.getCode() == KeyCode.ENTER) {
                String q = input.getText() == null ? "" : input.getText().trim();
                if (!q.isEmpty() && onSearchSubmit != null) onSearchSubmit.accept(q);
                hide();
            }
        });

        results = new ListView<>();
        results.setFocusTraversable(false);

        VBox root = new VBox(10, input, results);
        root.setPadding(new Insets(14));
        root.setAlignment(Pos.CENTER_LEFT);
        root.getStyleClass().add("search-root"); // style global de la box

        Scene scene = new Scene(root, 680, 320);
        scene.setFill(null);

        // Charger notre CSS
        scene.getStylesheets().add(
            getClass().getResource("/dark.css").toExternalForm()
        );

        stage.setScene(scene);
        stage.setOnShown(e -> stage.centerOnScreen());
    }

    public void show() {
        Platform.runLater(() -> {
            if (!stage.isShowing()) stage.show();
            stage.toFront();
            input.requestFocus();
            input.selectAll();
        });
    }

    public void hide() { Platform.runLater(stage::hide); }

    public void toggle() {
        Platform.runLater(() -> {
            if (stage.isShowing()) hide();
            else show();
        });
    }

    // callback quand l’utilisateur presse Entrée
    public void setOnSearchSubmit(Consumer<String> onSearchSubmit) {
        this.onSearchSubmit = onSearchSubmit;
    }
}
