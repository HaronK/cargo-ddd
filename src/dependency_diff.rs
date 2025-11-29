use crate::crate_diff_info::CrateDiffInfo;

/// Complete diff information between crate's 2 versions and all its nested dependencies
pub struct DependencyDiff {
    /// Direct dependency diff
    pub diff: CrateDiffInfo,
    /// Removed nested dependencies
    pub removed_deps: Vec<CrateDiffInfo>,
    /// Added nested dependencies
    pub added_deps: Vec<CrateDiffInfo>,
    /// Updated nested dependencies
    pub updated_deps: Vec<CrateDiffInfo>,
}

impl DependencyDiff {
    pub fn print(&self) {
        self.diff.print(1);

        if self.removed_deps.is_empty()
            && self.added_deps.is_empty()
            && self.updated_deps.is_empty()
        {
            return;
        }

        println!("    Nested dependency diffs:");

        // print removed, updated and added diffs properly
        if !self.removed_deps.is_empty() {
            println!("      Removed:");
            for dep in &self.removed_deps {
                dep.print(4);
            }
        }

        if !self.added_deps.is_empty() {
            println!("      Added:");
            for dep in &self.added_deps {
                dep.print(4);
            }
        }

        if !self.updated_deps.is_empty() {
            println!("      Updated:");
            for dep in &self.updated_deps {
                dep.print(4);
            }
        }
    }
}
