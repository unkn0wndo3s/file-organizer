package fr.unkn0wndo3s.app;

import java.awt.AWTException;
import java.awt.BasicStroke;
import java.awt.Color;
import java.awt.Graphics2D;
import java.awt.Image;
import java.awt.MenuItem;
import java.awt.PopupMenu;
import java.awt.RenderingHints;
import java.awt.SystemTray;
import java.awt.TrayIcon;
import java.awt.image.BufferedImage;

import fr.unkn0wndo3s.ui.SearchWindow;
import javafx.application.Platform;

public final class TrayUtil {
    private TrayUtil() {}

    public static void installTray(SearchWindow searchWindow, Runnable onQuit) {
        if (!SystemTray.isSupported()) {
            System.err.println("SystemTray non supporté sur cette machine.");
            return;
        }

        // Petite icône 16x16 générée à la volée (évite de gérer un fichier .ico)
        Image icon = makeIcon();

        PopupMenu menu = new PopupMenu();

        MenuItem open = new MenuItem("Ouvrir (Ctrl+Espace)");
        open.addActionListener(e -> Platform.runLater(searchWindow::show));
        menu.add(open);

        MenuItem hide = new MenuItem("Masquer");
        hide.addActionListener(e -> Platform.runLater(searchWindow::hide));
        menu.add(hide);

        menu.addSeparator();

        MenuItem quit = new MenuItem("Quitter");
        quit.addActionListener(e -> {
            if (onQuit != null) onQuit.run();
            SystemTray.getSystemTray().remove(findTray(icon));
            Platform.exit();
        });
        menu.add(quit);

        TrayIcon trayIcon = new TrayIcon(icon, "File Organizer", menu);
        trayIcon.setImageAutoSize(true);
        trayIcon.addActionListener(e -> Platform.runLater(searchWindow::toggle)); // clic simple = toggle

        try {
            SystemTray.getSystemTray().add(trayIcon);
        } catch (AWTException ex) {
            ex.printStackTrace();
        }
    }

    private static TrayIcon findTray(Image icon) {
        for (TrayIcon ti : SystemTray.getSystemTray().getTrayIcons()) {
            if (ti.getImage() == icon) return ti;
        }
        return null;
    }

    private static Image makeIcon() {
        int s = 16;
        BufferedImage img = new BufferedImage(s, s, BufferedImage.TYPE_INT_ARGB);
        Graphics2D g = img.createGraphics();
        g.setRenderingHint(RenderingHints.KEY_ANTIALIASING, RenderingHints.VALUE_ANTIALIAS_ON);
        // pastille sombre
        g.setColor(new Color(33, 33, 33, 255));
        g.fillRoundRect(0, 0, s, s, 6, 6);
        // loupe blanche minimaliste
        g.setStroke(new BasicStroke(1.7f));
        g.setColor(Color.WHITE);
        g.drawOval(3, 3, 8, 8);
        g.drawLine(9, 9, 13, 13);
        g.dispose();
        return img;
    }
}
