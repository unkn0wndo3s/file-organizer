package fr.unkn0wndo3s.core.planner;

import java.nio.file.Path;
import java.util.EnumMap;
import java.util.Map;

import fr.unkn0wndo3s.core.rules.Category;

public final class PlannerConfig {
    private final Map<Category, Path> roots = new EnumMap<>(Category.class);

    public PlannerConfig(Path documents, Path pictures, Path videos, Path music, Path apps, Path code, Path other) {
        roots.put(Category.DOCUMENTS, documents);
        roots.put(Category.IMAGES,    pictures);
        roots.put(Category.VIDEOS,    videos);
        roots.put(Category.AUDIO,     music);
        roots.put(Category.APPS,      apps);
        roots.put(Category.CODE,      code);
        roots.put(Category.ARCHIVES,  documents.resolve("Archives"));
        roots.put(Category.OTHER,     other);
    }

    public Path dirFor(Category c) { return roots.get(c); }

    public static PlannerConfig windowsDefaults(Path userHome) {
        return new PlannerConfig(
                userHome.resolve("Documents"),
                userHome.resolve("Pictures"),
                userHome.resolve("Videos"),
                userHome.resolve("Music"),
                userHome.resolve("Downloads").resolve("Apps"),
                userHome.resolve("Documents").resolve("Code"),
                userHome.resolve("Documents").resolve("Other")
        );
    }
}
