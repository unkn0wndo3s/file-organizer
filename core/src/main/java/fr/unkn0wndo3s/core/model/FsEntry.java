package fr.unkn0wndo3s.core.model;

import java.nio.file.Path;

public record FsEntry(
        Path path,
        String name,
        boolean isDirectory,
        String extensionLower,
        long sizeBytes,
        long lastModifiedEpochMillis
) {}
