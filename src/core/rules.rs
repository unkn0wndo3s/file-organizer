use std::collections::HashMap;

/// Destination category a file is classified into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Category {
    Documents,
    Images,
    Videos,
    Audio,
    Archives,
    Code,
    Apps,
    Other,
}

/// Maps lowercase file extensions to their category.
pub struct RuleSet {
    by_extension: HashMap<&'static str, Category>,
}

impl RuleSet {
    pub fn new() -> Self {
        let mut by_extension = HashMap::new();

        let mut map = |category: Category, extensions: &[&'static str]| {
            for extension in extensions {
                by_extension.insert(*extension, category);
            }
        };

        map(Category::Documents, &["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "odt", "ods", "txt", "rtf", "csv", "md", "tex"]);
        map(Category::Images, &["jpg", "jpeg", "png", "gif", "bmp", "webp", "tiff", "svg", "heic", "psd", "ai"]);
        map(Category::Videos, &["mp4", "mkv", "mov", "avi", "wmv", "flv", "webm", "m4v"]);
        map(Category::Audio, &["mp3", "wav", "flac", "aac", "ogg", "m4a"]);
        map(Category::Archives, &["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso"]);
        map(Category::Code, &["java", "kt", "scala", "py", "js", "ts", "tsx", "jsx", "json", "xml", "yml", "yaml", "ini", "cfg", "toml", "html", "css", "c", "cpp", "h", "hpp", "rs", "go", "php", "rb", "sh", "bat", "ps1", "sql"]);
        map(Category::Apps, &["exe", "msi", "msix", "apk"]);

        Self { by_extension }
    }

    /// Classifies an already lowercased extension, without its leading dot.
    pub fn classify(&self, extension_lower: &str) -> Category {
        if extension_lower.trim().is_empty() {
            return Category::Other;
        }
        self.by_extension.get(extension_lower).copied().unwrap_or(Category::Other)
    }
}

impl Default for RuleSet {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_extensions_map_to_their_category() {
        let rules = RuleSet::new();
        assert_eq!(rules.classify("pdf"), Category::Documents);
        assert_eq!(rules.classify("rs"), Category::Code);
        assert_eq!(rules.classify("mkv"), Category::Videos);
        assert_eq!(rules.classify("exe"), Category::Apps);
    }

    #[test]
    fn unknown_and_empty_extensions_fall_back_to_other() {
        let rules = RuleSet::new();
        assert_eq!(rules.classify("qwerty"), Category::Other);
        assert_eq!(rules.classify(""), Category::Other);
        assert_eq!(rules.classify("   "), Category::Other);
    }
}
