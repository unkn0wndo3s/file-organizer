package fr.unkn0wndo3s.core.service;

import java.io.IOException;
import java.nio.file.Path;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.List;

import fr.unkn0wndo3s.core.model.FilePlan;
import fr.unkn0wndo3s.core.model.FileRecord;
import fr.unkn0wndo3s.core.planner.FilePlanner;
import fr.unkn0wndo3s.core.scanner.FileScanner;

public final class PreArrangeService {
    private final FileScanner scanner;
    private final FilePlanner planner;

    public PreArrangeService(FileScanner scanner, FilePlanner planner) {
        this.scanner = scanner;
        this.planner = planner;
    }

    public List<FilePlan> preview(List<Path> roots) throws IOException {
        List<FileRecord> found = scanner.scan(roots);
        List<FilePlan> plans = new ArrayList<>(found.size());
        for (FileRecord r : found) {
            plans.add(planner.plan(r));
        }
        // tri : plus récents en premier
        plans.sort(Comparator.comparingLong((FilePlan p) -> p.record().lastModifiedEpochMillis()).reversed());
        return plans;
    }

    public void previewStream(List<Path> roots, java.util.function.Consumer<FilePlan> onPlan) throws IOException {
        scanner.scanStream(roots, rec -> {
            FilePlan p = planner.plan(rec);
            onPlan.accept(p);
        });
    }
    
}
