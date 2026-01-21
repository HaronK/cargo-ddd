use semver::Version;

#[derive(Debug, Clone)]
pub struct CrateInfo {
    pub version: Option<Version>,
    pub repository: Option<String>,
}
