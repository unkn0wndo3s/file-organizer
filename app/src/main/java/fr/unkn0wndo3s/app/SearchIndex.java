package fr.unkn0wndo3s.app;

import java.nio.file.Path;
import java.util.Collection;
import java.util.Map;
import java.util.concurrent.ConcurrentHashMap;

public final class SearchIndex {
    private final Map<String, Path> byName = new ConcurrentHashMap<>();

    public void clear() { byName.clear(); }

    public void add(Path p) {
        if (p == null) return;
        var name = p.getFileName() == null ? null : p.getFileName().toString();
        if (name != null && !name.isBlank()) byName.put(name, p);
    }

    public void addAll(Collection<Path> paths) {
        if (paths == null) return;
        for (Path p : paths) add(p);
    }

    public Collection<String> names() { return byName.keySet(); }

    public Path resolve(String name) { return byName.get(name); }
}
