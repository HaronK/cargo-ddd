use std::collections::HashMap;

use crate::dependency_diff::DependencyDiff;

/// Diff report for all requested crates
pub struct DiffReport {
    /// Dependency diffs per workspace target
    pub dependency_diffs: HashMap<String, Vec<DependencyDiff>>,
}
