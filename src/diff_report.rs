use std::collections::HashMap;

use crate::dependency_diff::DependencyDiff;

/// Diff report for all requested crates
pub struct DiffReport {
    /// Dependency diffs per workspace target
    pub dependency_diffs: HashMap<String, Vec<DependencyDiff>>,
}

impl DiffReport {
    pub fn print(&self) {
        for (target_name, diffs) in &self.dependency_diffs {
            if target_name.is_empty() {
                println!("Default dependencies:");
            } else {
                println!("{target_name} dependencies:");
            }

            for diff in diffs {
                diff.print();
            }
        }
    }
}
