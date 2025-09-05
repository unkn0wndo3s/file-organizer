package fr.unkn0wndo3s.core.rules;

import java.util.HashMap;
import java.util.Map;

public final class RuleSet {
    private final Map<String, Category> byExt = new HashMap<>();

    public RuleSet() {
        map(Category.DOCUMENTS, "pdf","doc","docx","xls","xlsx","ppt","pptx","odt","ods","txt","rtf","csv","md","tex");
        map(Category.IMAGES, "jpg","jpeg","png","gif","bmp","webp","tiff","svg","heic","psd","ai");
        map(Category.VIDEOS, "mp4","mkv","mov","avi","wmv","flv","webm","m4v");
        map(Category.AUDIO, "mp3","wav","flac","aac","ogg","m4a");
        map(Category.ARCHIVES, "zip","rar","7z","tar","gz","bz2","xz","iso");
        map(Category.CODE, "java","kt","scala","py","js","ts","tsx","jsx","json","xml","yml","yaml","ini","cfg","toml","html","css","c","cpp","h","hpp","rs","go","php","rb","sh","bat","ps1","sql");
        map(Category.APPS, "exe","msi","msix","apk");
    }

    private void map(Category cat, String... exts) {
        for (String e : exts) byExt.put(e, cat);
    }

    public Category classify(String extensionLower) {
        if (extensionLower == null || extensionLower.isBlank()) return Category.OTHER;
        return byExt.getOrDefault(extensionLower, Category.OTHER);
    }
}
