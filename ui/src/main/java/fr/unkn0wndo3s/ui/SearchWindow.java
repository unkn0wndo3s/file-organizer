// ui/src/main/java/fr/unkn0wndo3s/ui/SearchWindow.java
package fr.unkn0wndo3s.ui;

import java.util.Collection;
import java.util.Objects;
import java.util.function.Consumer;

import javafx.application.Platform;
import javafx.collections.FXCollections;
import javafx.collections.ObservableList;
import javafx.collections.transformation.FilteredList;
import javafx.geometry.Insets;
import javafx.geometry.Pos;
import javafx.scene.Scene;
import javafx.scene.control.ContextMenu;
import javafx.scene.control.ListView;
import javafx.scene.control.MenuItem;
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

    private final ObservableList<String> master = FXCollections.observableArrayList();
    private final FilteredList<String> filtered = new FilteredList<>(master, s -> true);

    private Consumer<String> onQuerySubmit;
    private Consumer<String> onActivateItem;
    private Consumer<String> onContextMenuItem;

    public SearchWindow() {
        stage = new Stage(StageStyle.UNDECORATED);
        stage.initModality(Modality.NONE);
        stage.setAlwaysOnTop(true);

        results = new ListView<>(filtered);
        results.setFocusTraversable(false);
        results.setOnMouseClicked(e -> {
            if (e.getClickCount() >= 2) {
                String sel = results.getSelectionModel().getSelectedItem();
                if (sel != null && onActivateItem != null) {
                    onActivateItem.accept(sel);
                    hide();
                }
            }
        });

        ContextMenu menu = new ContextMenu();
        MenuItem openFolder = new MenuItem("Ouvrir l’emplacement du fichier");
        openFolder.setOnAction(ev -> {
            String sel = results.getSelectionModel().getSelectedItem();
            if (sel != null && onContextMenuItem != null) onContextMenuItem.accept(sel);
        });
        menu.getItems().add(openFolder);
        results.setContextMenu(menu);

        input = new TextField();
        input.setPromptText("Write to search");
        input.getStyleClass().add("search-input");

        input.textProperty().addListener((obs, oldV, v) -> {
            final String q = v == null ? "" : v.trim().toLowerCase();
            filtered.setPredicate(name -> q.isEmpty() || (name != null && name.toLowerCase().contains(q)));
            if (!filtered.isEmpty()) results.getSelectionModel().select(0);
        });

        input.setOnKeyPressed(e -> {
            if (e.getCode() == KeyCode.ESCAPE) hide();
            if (e.getCode() == KeyCode.DOWN) {
                results.getSelectionModel().selectNext();
                results.scrollTo(results.getSelectionModel().getSelectedIndex());
                e.consume();
            }
            if (e.getCode() == KeyCode.UP) {
                results.getSelectionModel().selectPrevious();
                results.scrollTo(results.getSelectionModel().getSelectedIndex());
                e.consume();
            }
            if (e.getCode() == KeyCode.ENTER) {
                String q = input.getText() == null ? "" : input.getText().trim();
                String sel = results.getSelectionModel().getSelectedItem();
                if (sel != null && onActivateItem != null) {
                    onActivateItem.accept(sel);
                    hide();
                } else if (!q.isEmpty() && onQuerySubmit != null) {
                    onQuerySubmit.accept(q);
                    hide();
                }
            }
        });

        VBox root = new VBox(10, input, results);
        root.setPadding(new Insets(14));
        root.setAlignment(Pos.CENTER_LEFT);
        root.getStyleClass().add("search-root");

        Scene scene = new Scene(root, 680, 320);
        scene.setFill(null);
        scene.getStylesheets().add(
            Objects.requireNonNull(getClass().getResource("/dark.css")).toExternalForm()
        );
        stage.setScene(scene);
        stage.setOnShown(e -> stage.centerOnScreen());
    }

    public void setItems(Collection<String> names) {
        Platform.runLater(() -> {
            master.setAll(names == null ? java.util.List.of() : names);
            if (!filtered.isEmpty()) results.getSelectionModel().select(0);
        });
    }

    public void refreshKeepingFilter() {
        Platform.runLater(() -> {
            final String q = input.getText();
            input.setText(q);
            input.positionCaret(q == null ? 0 : q.length());
            if (!filtered.isEmpty()) results.getSelectionModel().select(0);
        });
    }

    public void setOnQuerySubmit(Consumer<String> cb)   { this.onQuerySubmit = cb; }
    public void setOnActivateItem(Consumer<String> cb)  { this.onActivateItem = cb; }
    public void setOnContextMenuItem(Consumer<String> cb) { this.onContextMenuItem = cb; }

    public void show() {
        Platform.runLater(() -> {
            if (!stage.isShowing()) stage.show();
            stage.toFront();
            input.requestFocus();
            input.selectAll();
        });
    }
    public void hide()   { Platform.runLater(stage::hide); }
    public void toggle() { Platform.runLater(() -> { if (stage.isShowing()) hide(); else show(); }); }
}
