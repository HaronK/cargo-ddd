use semver::Version;

/// Diff request parsed from the cli arguments
#[derive(Debug, Clone)]
pub struct CrateDiffRequest {
    /// Crate name
    pub crate_name: String,
    /// Initial version info. Either exact version or version from the Cargo.toml
    pub from_version: Option<Version>,
    /// Target version info. Either exact version or version from the registry (crates.io)
    pub to_version: Option<Version>,
}
