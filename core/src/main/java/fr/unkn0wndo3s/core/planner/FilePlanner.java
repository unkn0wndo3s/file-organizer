package fr.unkn0wndo3s.core.planner;

import java.nio.file.Path;

import fr.unkn0wndo3s.core.model.FilePlan;
import fr.unkn0wndo3s.core.model.FileRecord;
import fr.unkn0wndo3s.core.rules.Category;
import fr.unkn0wndo3s.core.rules.RuleSet;

public final class FilePlanner {
    private final RuleSet rules;
    private final PlannerConfig cfg;

    public FilePlanner(RuleSet rules, PlannerConfig cfg) {
        this.rules = rules;
        this.cfg = cfg;
    }

    public FilePlan plan(FileRecord rec) {
        Category cat = rules.classify(rec.extensionLower());
        Path destDir = cfg.dirFor(cat);
        Path destPath = destDir.resolve(safeName(rec.name()));
        return new FilePlan(rec, cat, destDir, destPath);
    }

    private String safeName(String name) {
        // Simple normalisation (évite collisions à gérer plus tard)
        return name.replaceAll("[\\\\/:*?\"<>|]", "_");
    }
}
