use crate::core::model::FilePlan;
use crate::core::planner::FilePlanner;
use crate::core::scanner::FileScanner;
use std::path::Path;

/// Builds move plans without touching the file system, so the result can be
/// reviewed before anything is applied.
pub struct PreArrangeService {
    scanner: FileScanner,
    planner: FilePlanner,
}

impl PreArrangeService {
    pub fn new(scanner: FileScanner, planner: FilePlanner) -> Self {
        Self { scanner, planner }
    }

    /// Plans every file under `roots`, most recently modified first.
    pub fn preview(&self, roots: &[impl AsRef<Path>]) -> Vec<FilePlan> {
        let mut plans: Vec<FilePlan> =
            self.scanner.scan(roots).into_iter().map(|record| self.planner.plan(record)).collect();

        plans.sort_by(|left, right| right.record.last_modified.cmp(&left.record.last_modified));
        plans
    }

    /// Streams a plan for each file under `roots`, in scan order.
    pub fn preview_stream(&self, roots: &[impl AsRef<Path>], mut on_plan: impl FnMut(FilePlan)) {
        self.scanner.scan_stream(roots, |record| on_plan(self.planner.plan(record)));
    }
}
