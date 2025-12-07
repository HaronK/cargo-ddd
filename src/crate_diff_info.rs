use semver::Version;

/// The crate diff information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CrateDiffInfo {
    /// Crate name
    pub name: String,
    /// Initial crate version
    pub from_version: Option<Version>,
    /// Initial crate repository commit hash
    pub from_hash: Option<String>,
    /// Target crate version
    pub to_version: Option<Version>,
    /// Target crate repository commit hash
    pub to_hash: Option<String>,
    /// Crate repository path
    pub repository: Option<String>,
}
