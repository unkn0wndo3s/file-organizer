package fr.unkn0wndo3s.app;

import javafx.application.Platform;
import javafx.geometry.Insets;
import javafx.scene.Scene;
import javafx.scene.control.TextArea;
import javafx.scene.layout.StackPane;
import javafx.stage.Stage;
import javafx.stage.StageStyle;

public class LogWindow {
    private final Stage stage;
    private final TextArea area;

    public LogWindow() {
        stage = new Stage(StageStyle.UTILITY);
        stage.setTitle("Console");
        area = new TextArea();
        area.setEditable(false);
        area.setWrapText(false);
        var root = new StackPane(area);
        root.setPadding(new Insets(8));
        stage.setScene(new Scene(root, 720, 420));
    }

    public void append(String line) {
        Platform.runLater(() -> {
            area.appendText(line + System.lineSeparator());
            area.setScrollTop(Double.MAX_VALUE);
        });
    }

    public void show() {
        Platform.runLater(() -> {
            if (!stage.isShowing()) stage.show();
            stage.toFront();
        });
    }

    public void toggle() {
        Platform.runLater(() -> {
            if (stage.isShowing()) stage.hide();
            else show();
        });
    }
}
