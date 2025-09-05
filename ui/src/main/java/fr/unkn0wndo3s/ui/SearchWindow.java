package fr.unkn0wndo3s.ui;

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

    public SearchWindow() {
        stage = new Stage(StageStyle.UNDECORATED);
        stage.initModality(Modality.NONE);
        stage.setAlwaysOnTop(true);

        input = new TextField();
        input.setPromptText("Tape pour chercher…");
        input.setOnKeyPressed(e -> {
            if (e.getCode() == KeyCode.ESCAPE)
                hide();
            if (e.getCode() == KeyCode.ENTER) {
                hide();
            }
        });

        results = new ListView<>();
        results.setMouseTransparent(true);
        results.getItems().addAll("Résultat (démo)", "Brancher le moteur de recherche ici");

        VBox root = new VBox(10, input, results);
        root.setPadding(new Insets(14));
        root.setAlignment(Pos.CENTER_LEFT);
        root.setStyle("-fx-background-color: #1f1f1f; -fx-background-radius: 8;");

        Scene scene = new Scene(root, 600, 220);
        scene.setFill(null);
        stage.setScene(scene);
        stage.setOnShown(e -> stage.centerOnScreen());
    }

    public void show() {
        Platform.runLater(() -> {
            if (!stage.isShowing()) {
                stage.show();
            }
            stage.toFront();
            input.requestFocus();
            input.selectAll();
        });
    }

    public void hide() {
        Platform.runLater(stage::hide);
    }

    public void toggle() {
        Platform.runLater(() -> {
            if (stage.isShowing())
                hide();
            else
                show();
        });
    }
}
