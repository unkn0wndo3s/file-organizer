package fr.unkn0wndo3s.core.model;

import java.nio.file.Path;

public record FileRecord(
        Path path,
        String name,
        String extensionLower,
        long sizeBytes,
        long lastModifiedEpochMillis
) {}
