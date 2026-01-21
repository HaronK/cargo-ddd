use cargo_metadata::PackageId;
use semver::Version;

use crate::package_source::PackageSource;

/// Parsed PackageId information returned by cargo metadata command
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PackageIdInfo {
    /// Package source: registry, path or other
    pub source: PackageSource,
    /// Crate URL on the rust-lang GitHub repository
    pub path: String,
    /// Crate name
    pub name: String,
    /// Crate version
    pub version: Version,
}

impl PackageIdInfo {
    pub fn from_package_id(pkg_id: &PackageId) -> Option<Self> {
        let Some((source, tail)) = pkg_id.repr.split_once('+') else {
            eprintln!("Cannot extract source: {pkg_id}");
            return None;
        };
        let Some((path, tail)) = tail.split_once('#') else {
            eprintln!("Cannot extract path: {pkg_id}");
            return None;
        };
        let (name, version) = if let Some(name_version) = tail.split_once('@') {
            name_version
        } else {
            let Some((_, name)) = path.rsplit_once("/") else {
                eprintln!("Cannot extract package name from: {path}");
                return None;
            };
            (name, tail)
        };
        // Git repositories can have branch part so we remove it. Example: tauri-plugin-trafficlights-positioner?branch=v2
        let name = if source == "git"
            && let Some((name, _)) = name.split_once('?')
        {
            name
        } else {
            name
        };
        let version = match Version::parse(version) {
            anyhow::Result::Ok(version) => version,
            Err(err) => {
                eprintln!("Cannot parse version: {version}. Error: {err}");
                return None;
            }
        };

        Some(PackageIdInfo {
            source: source.into(),
            path: path.into(),
            name: name.into(),
            version,
        })
    }

    pub fn parse_source(pkg_id: &PackageId) -> Option<PackageSource> {
        let Some((source, _tail)) = pkg_id.repr.split_once('+') else {
            eprintln!("Cannot extract source: {pkg_id}");
            return None;
        };
        Some(source.into())
    }
}

impl From<&PackageIdInfo> for PackageId {
    fn from(value: &PackageIdInfo) -> Self {
        Self {
            repr: format!(
                "{}+{}#{}@{}",
                value.source.as_str(),
                value.path,
                value.name,
                value.version
            ),
        }
    }
}
