use std::path::PathBuf;

use anyhow::{Result, anyhow};
use semver::Version;

use crate::cargo_runner::CargoRunner;

/// Manager for the local cargo registry crate sorces
pub struct RegistryManager {
    /// Path to the current user cargo registry source folder
    registry_path: PathBuf,
}

impl RegistryManager {
    pub fn new(registry_path: Option<PathBuf>) -> Result<Self> {
        if let Some(registry_path) = registry_path {
            return Ok(Self { registry_path });
        }

        // Try to find registry path
        let cargo_home = match std::env::var("CARGO_HOME") {
            Ok(cargo_home) => cargo_home,
            Err(_) => format!("{}/.cargo", std::env::var("HOME")?),
        };

        let cargo_home_path = PathBuf::from(&cargo_home);

        if !std::fs::exists(&cargo_home_path)? {
            return Err(anyhow!("Cargo path doesn't exist: {cargo_home}"));
        }

        // NOTE: this is a very 'hacky' way to get local cargo registry source folder
        let registry_src_dir = cargo_home_path.join("registry").join("src");
        let entries = std::fs::read_dir(registry_src_dir)?;
        let mut buf = vec![];

        // Find registry folder name
        for entry in entries {
            let entry = entry?;
            let meta = entry.metadata()?;

            if meta.is_dir() {
                let Ok(file_name) = entry.file_name().into_string() else {
                    eprintln!("Cannot read file name: {:?}", entry.file_name());
                    continue;
                };
                // By default we are using sources from the crates.io registry
                if file_name.starts_with("index.crates.io-") {
                    buf.push(entry.path());
                }
            }
        }

        match buf.as_slice() {
            [] => Err(anyhow!("Cannot get crates.io registry source code path")),
            [first, remaider @ ..] => {
                if !remaider.is_empty() {
                    eprintln!(
                        "[WARN] There are {} registry sources. First fill be used: {buf:#?}",
                        buf.len()
                    );
                }
                Ok(Self {
                    registry_path: first.clone(),
                })
            }
        }
    }

    /// Get path to the crate source code in the local cargo registry
    pub fn get_crate_path(&self, crate_name: &str, version: &Version) -> PathBuf {
        self.registry_path.join(format!("{crate_name}-{version}"))
    }

    /// Crate version commit hash from the '.cargo_vcs_info.json' file in the crate source folder in the loacl registry
    pub fn get_crate_hash(&self, crate_name: &str, version: &Version) -> Option<String> {
        let cargo_runner = CargoRunner::new(None);

        // Run 'cargo info' for the specific version of the crate to guarantee it's in the local registry
        if let Err(err) = cargo_runner.run("info", [format!("{crate_name}@{version}")]) {
            eprintln!("Cannot get '{crate_name}' crate info. Error: {err}");
            return None;
        }

        let vcs_info_path = self
            .registry_path
            .join(format!("{crate_name}-{version}"))
            .join(".cargo_vcs_info.json");

        match std::fs::exists(&vcs_info_path) {
            Ok(file_exists) => {
                if !file_exists {
                    eprintln!(
                        "Crate doesn't contain .cargo_vcs_info.json. Commit hash is not available for: {crate_name}@{version}"
                    );
                    return None;
                }
            }
            Err(err) => {
                // TODO: extract commit hash from the other sources
                eprintln!(
                    "Cannot access .cargo_vcs_info.json file of the '{crate_name}@{version}' repository. Error: {err}"
                );
                return None;
            }
        }

        let hash_data = match std::fs::read_to_string(&vcs_info_path) {
            Ok(hash_data) => hash_data,
            Err(err) => {
                // TODO: extract commit hash from the other sources
                eprintln!(
                    "Cannot read '{crate_name}@{version}' crate commit hash from the '{vcs_info_path:?}' file. Error: {err}"
                );
                return None;
            }
        };

        // Do not parse Json. Just read hash from the line
        if let Some(hash) = hash_data
            .lines()
            .find_map(|l| l.trim().strip_prefix("\"sha1\": \""))
        {
            // strip last " symbol
            Some(hash[..hash.len() - 1].into())
        } else {
            // TODO: extract commit hash from the other sources
            eprintln!("Cannot get hash of the '{crate_name}' crate:\n{hash_data}");
            None
        }
    }

    /// Extract crate version and repository from the output of the 'cargo info' command.
    /// This will automatically download crate and its sources into the local cargo registry.
    pub fn get_crate_info(
        &self,
        crate_name: &str,
        version: Option<&Version>,
    ) -> (Option<Version>, Option<String>) {
        let cargo_runner = CargoRunner::new(None);
        let crate_desc = if let Some(version) = version {
            format!("{crate_name}@{version}")
        } else {
            crate_name.into()
        };
        let output = match cargo_runner.run("info", [&crate_desc]) {
            Ok(output) => output,
            Err(err) => {
                eprintln!("'cargo info {crate_desc}' command failed. Error: {err}");
                return (None, None);
            }
        };

        let version = output.lines().find_map(|l| {
            l.strip_prefix("version: ").and_then(|version_desc| {
                let version_str = if let Some((cur_version, latest_version)) =
                    version_desc.split_once(" (latest ")
                {
                    let version_str = if version.is_some() {
                        cur_version
                    } else {
                        &latest_version[..latest_version.len() - 1]
                    };
                    let cargo_runner = CargoRunner::new(None);
                    // load crate version into the local registry if it's not yet there
                    if let Err(err) =
                        cargo_runner.run("info", [format!("{crate_name}@{version_str}")])
                    {
                        eprintln!(
                            "'cargo info {crate_name}@{version_str}' command failed. Error: {err}"
                        );
                    }
                    version_str
                } else {
                    version_desc
                };

                match Version::parse(version_str) {
                    Ok(version) => Some(version),
                    Err(err) => {
                        eprintln!(
                            "Cannot parse '{crate_name}' version '{version_str:?}'. Error: {err}"
                        );
                        None
                    }
                }
            })
        });

        let repository = Self::repository_from_output(&output);
        if repository.is_none() {
            // TODO: Get repository in other way
            eprintln!("Cannot get repository of the '{crate_name}' crate:\n{output}");
        }

        (version, repository)
    }

    /// Extract crate repository from the output of the 'cargo info' command.
    /// This will automatically download crate and its sources into the local cargo registry.
    pub fn get_crate_repository(
        &self,
        crate_name: &str,
        version: Option<&Version>,
    ) -> Option<String> {
        let cargo_runner = CargoRunner::new(None);
        let crate_desc = if let Some(version) = version {
            format!("{crate_name}@{version}")
        } else {
            crate_name.into()
        };
        let output = match cargo_runner.run("info", [crate_desc]) {
            Ok(output) => output,
            Err(err) => {
                eprintln!("Cannot get '{crate_name}' crate info. Error: {err}");
                return None;
            }
        };
        let repository = Self::repository_from_output(&output);

        if repository.is_none() {
            // TODO: Get repository in other way
            eprintln!("Cannot get repository of the '{crate_name}' crate:\n{output}");
        }

        repository
    }

    fn repository_from_output(output: &str) -> Option<String> {
        output
            .lines()
            .find_map(|l| l.strip_prefix("repository: "))
            .map(|r| r.into())
            .or_else(|| {
                if let Some(homepage) = output.lines().find_map(|l| l.strip_prefix("homepage: "))
                    && homepage.starts_with("https://github.com/")
                {
                    let parts: Vec<_> = homepage.split('/').collect();
                    if parts.len() >= 5 {
                        return Some(parts[..5].join("/"));
                    }
                }
                // TODO: extract repository from ther sources
                None
            })
    }
}
