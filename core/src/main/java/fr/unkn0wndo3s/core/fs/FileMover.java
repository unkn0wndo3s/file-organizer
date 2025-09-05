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

    private static final Set<String> DOCS = set("txt","pdf","doc","docx","rtf","odt","xls","xlsx","csv","ppt","pptx","md","json","xml","yaml","yml");
    private static final Set<String> IMGS = set("jpg","jpeg","png","gif","bmp","tif","tiff","webp","heic","svg","ico");
    private static final Set<String> MUS  = set("mp3","wav","flac","aac","ogg","m4a","wma","opus");
    private static final Set<String> VIDS = set("mp4","mkv","avi","mov","wmv","webm","m4v");
    private static final Set<String> EXEC = set("exe", "msi", "iso", "jar", "bat", "cmd", "sh");
    private static final Set<String> ARCH = set("zip","rar","7z","tar","gz","bz2","xz");
    private static Set<String> set(String... exts) { return new HashSet<>(Arrays.asList(exts)); }

    public static String movePath(Path path) {
        try {
            if (Files.isDirectory(path)) {
                moveDirectory(path);
            } else if (Files.isRegularFile(path)) {
                moveFile(path);
                return "[move] moved : " + path;
            } else {
                return "[move] ignored (no file or folder) : " + path;
            }
        } catch (Exception e) {
            return "[move] error: " + e.getMessage();
        }
        return "[move] OK (folder) : " + path;
    }

    public static String moveFile(Path file) {
        if (file == null || !Files.isRegularFile(file)) return "";

        String ext = ext(file.getFileName().toString());
        String bucket = bucketFor(ext);
        if (bucket == null) {
            return "[move] ignored (unknown ext) : " + file;
        }

        Path home = Path.of(System.getProperty("user.home"));
        Path targetDir = home.resolve(bucket);

        try {
            Files.createDirectories(targetDir);
            Path target = uniqueTarget(targetDir, file.getFileName().toString());
            Files.move(file, target, StandardCopyOption.REPLACE_EXISTING);
            return "[move:file] " + file + "  ->  " + target;
        } catch (IOException e) {
            return "[move:file] fail: " + file + " : " + e.getMessage();
        }
    }

    public static String moveDirectory(Path dir) {
        if (dir == null || !Files.isDirectory(dir)) return "";

        Path home = Path.of(System.getProperty("user.home"));
        Path targetRoot = home.resolve("Folders");
        try {
            Files.createDirectories(targetRoot);
            Path target = uniqueTarget(targetRoot, dir.getFileName().toString());
            Files.move(dir, target);
            return "[move:dir] " + dir + "  ->  " + target;
        } catch (AtomicMoveNotSupportedException e) {
            try {
                Path target = uniqueTarget(targetRoot, dir.getFileName().toString());
                Files.move(dir, target);
                return "[move:dir] (non-atomic) " + dir + "  ->  " + target;
            } catch (IOException ex) {
                return "[move:dir] fail: " + dir + " : " + ex.getMessage();
            }
        } catch (FileAlreadyExistsException e) {
            return "[move:dir] already exists (unlikely because uniqueTarget) : " + e.getMessage();
        } catch (IOException e) {
            return "[move:dir] fail: " + dir + " : " + e.getMessage();
        }
    }
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
