package fr.unkn0wndo3s.core.model;

import java.nio.file.Path;

import fr.unkn0wndo3s.core.rules.Category;

public record FilePlan(
        FileRecord record,
        Category category,
        Path proposedDestinationDir,
        Path proposedDestinationPath
) {}
