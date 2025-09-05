// app/src/main/java/fr/unkn0wndo3s/app/SearchIndex.java
package fr.unkn0wndo3s.app;

import java.nio.file.Files;
import java.nio.file.LinkOption;
import java.nio.file.Path;
import java.nio.file.attribute.FileTime;
import java.util.Collection;
import java.util.Set;
import java.util.concurrent.ConcurrentHashMap;

public final class SearchIndex {
    private final ConcurrentHashMap<String, Set<Path>> byName = new ConcurrentHashMap<>();

    public void clear() { byName.clear(); }

    public void add(Path p) {
        if (p == null) return;
        var fn = p.getFileName();
        if (fn == null) return;
        String name = fn.toString();
        if (name.isBlank()) return;
        byName.computeIfAbsent(name, k -> ConcurrentHashMap.newKeySet()).add(p);
    }

    public void addAll(Collection<Path> paths) {
        if (paths == null) return;
        for (Path p : paths) add(p);
    }

    public void remove(Path p) {
        if (p == null) return;
        var fn = p.getFileName();
        if (fn == null) return;
        String name = fn.toString();
        var set = byName.get(name);
        if (set == null) return;
        set.remove(p);
        if (set.isEmpty()) byName.remove(name);
    }

    public Collection<String> names() { return byName.keySet(); }

    public Path resolveExact(String name) {
        if (name == null) return null;
        var set = byName.get(name);
        if (set == null || set.isEmpty()) return null;

        Path best = null;
        FileTime bestTime = null;

        for (Path p : set) {
            if (!Files.exists(p, LinkOption.NOFOLLOW_LINKS)) continue;
            FileTime t = null;
            try { t = Files.getLastModifiedTime(p, LinkOption.NOFOLLOW_LINKS); } catch (Exception ignore) {}
            if (best == null || isAfter(t, bestTime)) {
                best = p; bestTime = t;
            }
        }
        if (best != null) return best;
        return set.iterator().next();
    }

    private static boolean isAfter(FileTime a, FileTime b) {
        if (a == null && b == null) return false;
        if (a != null && b == null) return true;
        if (a == null) return false;
        return a.toMillis() > b.toMillis();
    }
}
