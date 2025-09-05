package fr.unkn0wndo3s.core.scanner;

import java.io.IOException;
import java.nio.file.FileVisitResult;
import java.nio.file.Files;
import java.nio.file.Path;
import java.nio.file.SimpleFileVisitor;
import java.nio.file.attribute.BasicFileAttributes;
import java.util.ArrayList;
import java.util.List;
import java.util.function.Consumer;

import fr.unkn0wndo3s.core.model.FileRecord;
import fr.unkn0wndo3s.core.model.FsEntry;

public final class FileScanner {

    // --- Scan récursif complet (déjà existant) ---
    public List<FileRecord> scan(List<Path> roots) throws IOException {
        List<FileRecord> out = new ArrayList<>();
        for (Path root : roots) {
            if (root == null || !Files.isDirectory(root)) continue;

            Files.walkFileTree(root, new SimpleFileVisitor<>() {
                @Override public FileVisitResult preVisitDirectory(Path dir, BasicFileAttributes attrs) {
                    try {
                        if (isHiddenOrSystem(dir)) return FileVisitResult.SKIP_SUBTREE;
                    } catch (IOException ignored) {}
                    return FileVisitResult.CONTINUE;
                }

                @Override public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) {
                    if (!attrs.isRegularFile()) return FileVisitResult.CONTINUE;
                    try {
                        if (isHiddenOrSystem(file)) return FileVisitResult.CONTINUE;
                    } catch (IOException ignored) {}

                    String name = file.getFileName().toString();
                    String ext = extensionLower(name);
                    long size = attrs.size();
                    long lm = attrs.lastModifiedTime().toMillis();

                    out.add(new FileRecord(file, name, ext, size, lm));
                    return FileVisitResult.CONTINUE;
                }
            });
        }
        return out;
    }

    // --- Streaming récursif complet ---
    public void scanStream(List<Path> roots, Consumer<FileRecord> onFile) throws IOException {
        for (Path root : roots) {
            if (root == null || !Files.isDirectory(root)) continue;
            Files.walkFileTree(root, new SimpleFileVisitor<>() {
                @Override public FileVisitResult preVisitDirectory(Path dir, BasicFileAttributes attrs) {
                    try { if (isHiddenOrSystem(dir)) return FileVisitResult.SKIP_SUBTREE; } catch (IOException ignored) {}
                    return FileVisitResult.CONTINUE;
                }
                @Override public FileVisitResult visitFile(Path file, BasicFileAttributes attrs) {
                    if (!attrs.isRegularFile()) return FileVisitResult.CONTINUE;
                    try { if (isHiddenOrSystem(file)) return FileVisitResult.CONTINUE; } catch (IOException ignored) {}
                    String name = file.getFileName().toString();
                    String ext = extensionLower(name);
                    onFile.accept(new FileRecord(file, name, ext, attrs.size(), attrs.lastModifiedTime().toMillis()));
                    return FileVisitResult.CONTINUE;
                }
            });
        }
    }

    // --- NEW: scan top-level only ---
    public List<FsEntry> scanTopLevel(List<Path> roots) throws IOException {
        List<FsEntry> out = new ArrayList<>();
        for (Path root : roots) {
            if (root == null || !Files.isDirectory(root)) continue;

            try (var stream = Files.list(root)) {
                stream.forEach(p -> {
                    try {
                        if (Files.isHidden(p)) return;
                        boolean isDir = Files.isDirectory(p);
                        var attrs = Files.readAttributes(p, BasicFileAttributes.class);
                        String name = p.getFileName().toString();
                        String ext = isDir ? "" : extensionLower(name);
                        long size = isDir ? 0L : attrs.size();
                        long lm = attrs.lastModifiedTime().toMillis();
                        out.add(new FsEntry(p, name, isDir, ext, size, lm));
                    } catch (Exception ignored) {}
                });
            }
        }
        return out;
    }

    // --- NEW: streaming top-level only ---
    public void scanTopLevelStream(List<Path> roots, Consumer<FsEntry> onEntry) throws IOException {
        for (Path root : roots) {
            if (root == null || !Files.isDirectory(root)) continue;

            try (var stream = Files.list(root)) {
                stream.forEach(p -> {
                    try {
                        if (Files.isHidden(p)) return;
                        boolean isDir = Files.isDirectory(p);
                        var attrs = Files.readAttributes(p, BasicFileAttributes.class);
                        String name = p.getFileName().toString();
                        String ext = isDir ? "" : extensionLower(name);
                        long size = isDir ? 0L : attrs.size();
                        long lm = attrs.lastModifiedTime().toMillis();
                        onEntry.accept(new FsEntry(p, name, isDir, ext, size, lm));
                    } catch (Exception ignored) {}
                });
            }
        }
    }

    private boolean isHiddenOrSystem(Path p) throws IOException {
        return Files.isHidden(p);
    }

    private String extensionLower(String name) {
        int i = name.lastIndexOf('.');
        if (i <= 0 || i == name.length() - 1) return "";
        return name.substring(i + 1).toLowerCase();
    }
}
