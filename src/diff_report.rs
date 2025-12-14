use indexmap::IndexMap;

use crate::dependency_diff::DependencyDiff;

/// Diff report for all requested crates
pub struct DiffReport {
    /// Dependency diffs per workspace target
    pub dependency_diffs: IndexMap<String, Vec<DependencyDiff>>,
}
