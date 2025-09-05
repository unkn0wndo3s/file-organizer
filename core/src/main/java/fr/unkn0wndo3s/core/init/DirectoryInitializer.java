package fr.unkn0wndo3s.core.init;

import java.io.IOException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.List;

public final class DirectoryInitializer {

    private DirectoryInitializer() {}

    /**
     * Crée les dossiers de base s'ils n'existent pas, + "Folders" à côté.
     * On travaille sous le HOME utilisateur :
     *   - Desktop, Downloads, Documents, Pictures, Videos, Music, Folders
     *
     * @return la liste des dossiers assurés (créés ou déjà existants)
     */
    public static List<Path> ensureBaseAndFolders(Path home) throws IOException {
        if (home == null) throw new IllegalArgumentException("home is null");

        // Noms standards (non localisés). Si ta machine est localisée,
        // on peut plus tard détecter via Known Folders (CSIDL) côté Win32.
        List<Path> targets = List.of(
                home.resolve("Desktop"),
                home.resolve("Downloads"),
                home.resolve("Documents"),
                home.resolve("Pictures"),
                home.resolve("Videos"),
                home.resolve("Music"),
                home.resolve("Folders"),
                home.resolve("Executables"),
                home.resolve("Archives")
        );

        List<Path> ensured = new ArrayList<>(targets.size());
        for (Path p : targets) {
            Files.createDirectories(p);
            ensured.add(p);
        }
        return ensured;
    }
}
