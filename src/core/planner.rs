use crate::core::model::{FilePlan, FileRecord};
use crate::core::rules::{Category, RuleSet};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Destination root for every category.
pub struct PlannerConfig {
    roots: HashMap<Category, PathBuf>,
}

impl PlannerConfig {
    pub fn new(
        documents: PathBuf,
        pictures: PathBuf,
        videos: PathBuf,
        music: PathBuf,
        apps: PathBuf,
        code: PathBuf,
        other: PathBuf,
    ) -> Self {
        let archives = documents.join("Archives");
        let roots = HashMap::from([
            (Category::Documents, documents),
            (Category::Images, pictures),
            (Category::Videos, videos),
            (Category::Audio, music),
            (Category::Apps, apps),
            (Category::Code, code),
            (Category::Archives, archives),
            (Category::Other, other),
        ]);
        Self { roots }
    }

    /// Layout used when nothing else is configured.
    pub fn defaults(home: &Path) -> Self {
        Self::new(
            home.join("Documents"),
            home.join("Pictures"),
            home.join("Videos"),
            home.join("Music"),
            home.join("Downloads").join("Apps"),
            home.join("Documents").join("Code"),
            home.join("Documents").join("Other"),
        )
    }

    pub fn dir_for(&self, category: Category) -> &Path {
        self.roots
            .get(&category)
            .map(PathBuf::as_path)
            .expect("every category has a configured root")
    }
}

/// Turns a scanned file into a proposed destination.
pub struct FilePlanner {
    rules: RuleSet,
    config: PlannerConfig,
}

impl FilePlanner {
    pub fn new(rules: RuleSet, config: PlannerConfig) -> Self {
        Self { rules, config }
    }

    pub fn plan(&self, record: FileRecord) -> FilePlan {
        let category = self.rules.classify(&record.extension_lower);
        let destination_dir = self.config.dir_for(category).to_path_buf();
        let destination_path = destination_dir.join(safe_name(&record.name));

        FilePlan {
            record,
            category,
            proposed_destination_dir: destination_dir,
            proposed_destination_path: destination_path,
        }
    }
}

/// Replaces the characters Windows forbids in a file name.
fn safe_name(name: &str) -> String {
    name.chars()
        .map(|c| if matches!(c, '\\' | '/' | ':' | '*' | '?' | '"' | '<' | '>' | '|') { '_' } else { c })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn characters_forbidden_by_windows_are_replaced() {
        assert_eq!(safe_name(r#"a/b\c:d*e?f"g<h>i|j"#), "a_b_c_d_e_f_g_h_i_j");
        assert_eq!(safe_name("already fine.txt"), "already fine.txt");
    }

    #[test]
    fn a_plan_targets_the_root_of_its_category() {
        let home = PathBuf::from("/home/user");
        let planner = FilePlanner::new(RuleSet::new(), PlannerConfig::defaults(&home));

        let plan = planner.plan(FileRecord {
            path: home.join("Downloads").join("photo.png"),
            name: "photo.png".to_string(),
            extension_lower: "png".to_string(),
            size_bytes: 0,
            last_modified: std::time::UNIX_EPOCH,
        });

        assert_eq!(plan.category, Category::Images);
        assert_eq!(plan.proposed_destination_path, home.join("Pictures").join("photo.png"));
    }
}
