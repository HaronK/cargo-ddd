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
