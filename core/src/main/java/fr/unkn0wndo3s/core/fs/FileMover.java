package fr.unkn0wndo3s.core.fs;

import java.io.IOException;
import java.nio.file.AtomicMoveNotSupportedException;
import java.nio.file.FileAlreadyExistsException;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.StandardCopyOption;
import java.util.Arrays;
import java.util.HashSet;
import java.util.Locale;
import java.util.Set;

public final class FileMover {

    private FileMover() {}

    // Buckets d’extensions
    private static final Set<String> DOCS = set("txt","pdf","doc","docx","rtf","odt","xls","xlsx","csv","ppt","pptx","md","json","xml","yaml","yml");
    private static final Set<String> IMGS = set("jpg","jpeg","png","gif","bmp","tif","tiff","webp","heic","svg","ico");
    private static final Set<String> MUS  = set("mp3","wav","flac","aac","ogg","m4a","wma","opus");
    private static final Set<String> VIDS = set("mp4","mkv","avi","mov","wmv","webm","m4v");
    private static final Set<String> EXEC = set("exe", "msi", "iso", "jar", "bat", "cmd", "sh");
    private static final Set<String> ARCH = set("zip","rar","7z","tar","gz","bz2","xz");
    private static Set<String> set(String... exts) { return new HashSet<>(Arrays.asList(exts)); }

    /** Si path = fichier → déplace par extension. Si path = dossier → déplace le dossier ENTIER vers ~/Folders. */
    public static String movePath(Path path) {
        try {
            if (Files.isDirectory(path)) {
                moveDirectory(path);         // <-- pas de récursion
            } else if (Files.isRegularFile(path)) {
                moveFile(path);
                return "[move] déplacé : " + path;
            } else {
                return "[move] ignoré (ni fichier ni dossier) : " + path;
            }
        } catch (Exception e) {
            return "[move] erreur: " + e.getMessage();
        }
        return "[move] OK (dossier) : " + path;
    }

    /** Déplace un fichier selon son extension ; inconnu => on ne bouge pas (reste dans Downloads). */
    public static String moveFile(Path file) {
        if (file == null || !Files.isRegularFile(file)) return "";

        String ext = ext(file.getFileName().toString());
        String bucket = bucketFor(ext);
        if (bucket == null) { // on ne bouge pas
            return "[move] ignoré (ext inconnue) : " + file;
        }

        Path home = Path.of(System.getProperty("user.home"));
        Path targetDir = home.resolve(bucket);

        try {
            Files.createDirectories(targetDir);
            Path target = uniqueTarget(targetDir, file.getFileName().toString());
            // move simple (rename). Si autre volume, ça lèvera et tu gèreras un copy/delete plus tard si tu veux.
            Files.move(file, target, StandardCopyOption.REPLACE_EXISTING);
            return "[move:file] " + file + "  ->  " + target;
        } catch (IOException e) {
            return "[move:file] échec: " + file + " : " + e.getMessage();
        }
    }

    /** Déplace un dossier ENTIER vers ~/Folders (sans parcourir son contenu). */
    public static String moveDirectory(Path dir) {
        if (dir == null || !Files.isDirectory(dir)) return "";

        Path home = Path.of(System.getProperty("user.home"));
        Path targetRoot = home.resolve("Folders");
        try {
            Files.createDirectories(targetRoot);
            Path target = uniqueTarget(targetRoot, dir.getFileName().toString());
            Files.move(dir, target); // pas de REPLACE_EXISTING sur dossier pour éviter l’écrasement foireux
            return "[move:dir] " + dir + "  ->  " + target;
        } catch (AtomicMoveNotSupportedException e) {
            // fallback si move atomique impossible
            try {
                Path target = uniqueTarget(targetRoot, dir.getFileName().toString());
                Files.move(dir, target);
                return "[move:dir] (non-atomique) " + dir + "  ->  " + target;
            } catch (IOException ex) {
                return "[move:dir] échec: " + dir + " : " + ex.getMessage();
            }
        } catch (FileAlreadyExistsException e) {
            return "[move:dir] existe déjà (improbable car uniqueTarget) : " + e.getMessage();
        } catch (IOException e) {
            return "[move:dir] échec: " + dir + " : " + e.getMessage();
        }
    }

    // ----- utils -----

    private static String bucketFor(String ext) {
        if (ext.isEmpty()) return null;
        if (DOCS.contains(ext)) return "Documents";
        if (IMGS.contains(ext)) return "Images";
        if (MUS.contains(ext))  return "Musics";
        if (VIDS.contains(ext)) return "Videos";
        if (EXEC.contains(ext)) return "Executables";
        if (ARCH.contains(ext)) return "Archives";
        return null;
    }

    private static String ext(String name) {
        int i = name.lastIndexOf('.');
        return (i < 0) ? "" : name.substring(i + 1).toLowerCase(Locale.ROOT);
    }

    /** Collision-safe: "foo" → "foo (1)", "foo (2)", … ; conserve .ext pour les fichiers. */
    private static Path uniqueTarget(Path dir, String filename) throws IOException {
        Path candidate = dir.resolve(filename);
        if (!Files.exists(candidate)) return candidate;

        String base = filename;
        String dotExt = "";
        int i = filename.lastIndexOf('.');
        if (i >= 0) {
            base = filename.substring(0, i);
            dotExt = filename.substring(i);
        }

        int n = 1;
        while (true) {
            Path alt = dir.resolve(base + " (" + n + ")" + dotExt);
            if (!Files.notExists(alt)) { n++; continue; }
            return alt;
        }
    }
}
