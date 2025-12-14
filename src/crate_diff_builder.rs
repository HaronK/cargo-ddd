use std::collections::HashMap;

use semver::Version;

use crate::cargo_meta::CargoMeta;
use crate::crate_diff_info::CrateDiffInfo;
use crate::crate_diff_request::CrateDiffRequest;
use crate::package_id_info::PackageIdInfo;
use crate::registry_manager::RegistryManager;

/// Generates diff information for the crate 2 versions
pub struct CrateDiffBuilder {
    registry_manager: RegistryManager,
    diff_rs: bool,
}

impl CrateDiffBuilder {
    pub fn new(registry_manager: RegistryManager, diff_rs: bool) -> Self {
        Self {
            registry_manager,
            diff_rs,
        }
    }

    /// Build diffs for the dependencies of the current crate that require update.
    /// Called when no crates are provided in the command line:
    ///   ddd
    pub fn build_from_crate(&self, cargo_meta: &CargoMeta) -> HashMap<String, Vec<CrateDiffInfo>> {
        let mut target_version_diffs = HashMap::new();
        let dependencies = cargo_meta.workspace_member_dependencies();

        for (target_name, deps) in dependencies {
            let mut diff_info = vec![];
            for dep in deps {
                let (latest_version, repository) =
                    self.registry_manager.get_crate_info(&dep.name, None);

                if self.diff_rs {
                    if let Some(latest_version) = &latest_version
                        && dep.version == *latest_version
                    {
                        continue;
                    }
                    diff_info.push(CrateDiffInfo {
                        name: dep.name,
                        from_version: Some(dep.version),
                        from_hash: None,
                        to_version: latest_version,
                        to_hash: None,
                        repository: None,
                    });
                } else if let Some(latest_version) = latest_version {
                    if dep.version != latest_version {
                        let from_hash = self
                            .registry_manager
                            .get_crate_hash(&dep.name, &dep.version);
                        let to_hash = self
                            .registry_manager
                            .get_crate_hash(&dep.name, &latest_version);

                        diff_info.push(CrateDiffInfo {
                            name: dep.name,
                            from_version: Some(dep.version),
                            from_hash,
                            to_version: Some(latest_version),
                            to_hash,
                            repository, // TODO: can a repository of the same crate change between versions?
                        });
                    }
                } else {
                    let from_hash = self
                        .registry_manager
                        .get_crate_hash(&dep.name, &dep.version);

                    diff_info.push(CrateDiffInfo {
                        name: dep.name,
                        from_version: Some(dep.version),
                        from_hash,
                        to_version: None,
                        to_hash: None,
                        repository, // TODO: can a repository of the same crate change between versions?
                    });
                }
            }
            target_version_diffs.insert(target_name, diff_info);
        }
        target_version_diffs
    }

    /// Build diffs for requested dependencies.
    /// Called when crates are provided in the command line and they require current crate info:
    ///   ddd serde@-1.0.223
    pub fn build_from_crate_deps(
        &self,
        crates: &[CrateDiffRequest],
        cargo_meta: &CargoMeta,
    ) -> HashMap<String, Vec<CrateDiffInfo>> {
        let mut target_version_diffs = HashMap::new();
        for pkg in crates {
            let mut from_versions = match &pkg.from_version {
                Some(version) => HashMap::from([("".to_string(), Some(version.clone()))]),
                None => {
                    // Take crate versions from Cargo.toml
                    cargo_meta
                        .get_dependency_info(&pkg.crate_name)
                        .into_iter()
                        .map(|(name, info)| (name, Some(info.version)))
                        .collect()
                }
            };

            let (to_version, repository) = if self.diff_rs {
                (
                    self.registry_manager
                        .get_crate_version(&pkg.crate_name, pkg.to_version.as_ref()),
                    None,
                )
            } else {
                self.registry_manager
                    .get_crate_info(&pkg.crate_name, pkg.to_version.as_ref())
            };

            if from_versions.is_empty() {
                // explicit/non-dependency crate
                from_versions.insert("".into(), None);
            }

            for (target_name, from_version) in from_versions {
                self.add_diff(
                    &target_name,
                    &pkg.crate_name,
                    from_version,
                    to_version.clone(),
                    repository.clone(),
                    &mut target_version_diffs,
                );
            }
        }
        target_version_diffs
    }

    /// Build diffs for requested dependencies.
    /// Called when crates are provided in the command line and they don't require current crate info:
    ///   ddd serde@1.0.223-1.0.226
    pub fn build_from_crates(
        &self,
        crates: &[CrateDiffRequest],
    ) -> HashMap<String, Vec<CrateDiffInfo>> {
        let mut target_version_diffs = HashMap::new();
        for pkg in crates {
            let (to_version, repository) = if self.diff_rs {
                (
                    self.registry_manager
                        .get_crate_version(&pkg.crate_name, pkg.to_version.as_ref()),
                    None,
                )
            } else {
                self.registry_manager
                    .get_crate_info(&pkg.crate_name, pkg.to_version.as_ref())
            };

            self.add_diff(
                "", // explicit/non-dependency crate
                &pkg.crate_name,
                pkg.from_version.clone(),
                to_version,
                repository,
                &mut target_version_diffs,
            );
        }
        target_version_diffs
    }

    /// Retrieves nested dependencies of 2 versions of the same crate and returns 3 lists of changes:
    /// - removed dependencies
    /// - added dependencies
    /// - updated dependencies
    pub fn build_nested_deps(
        &self,
        diff: &CrateDiffInfo,
    ) -> (Vec<CrateDiffInfo>, Vec<CrateDiffInfo>, Vec<CrateDiffInfo>) {
        let from_nested_packages = self.get_nested_packages(&diff.name, diff.from_version.as_ref());
        let mut to_nested_packages = self.get_nested_packages(&diff.name, diff.to_version.as_ref());

        let mut removed_deps = vec![];
        let mut updated_deps = vec![];

        for from_pkg in from_nested_packages {
            let pkg_idx = to_nested_packages
                .iter()
                .enumerate()
                .find_map(|(i, to_pkg)| {
                    if from_pkg.name == to_pkg.name {
                        Some(i)
                    } else {
                        None
                    }
                });
            let repository = if !self.diff_rs {
                self.registry_manager
                    .get_crate_repository(&from_pkg.name, Some(&from_pkg.version))
            } else {
                None
            };

            if let Some(index) = pkg_idx {
                let to_pkg = to_nested_packages.remove(index);

                if from_pkg.version != to_pkg.version {
                    let (from_hash, to_hash) = if !self.diff_rs {
                        let from_hash = self
                            .registry_manager
                            .get_crate_hash(&from_pkg.name, &from_pkg.version);
                        let to_hash = self
                            .registry_manager
                            .get_crate_hash(&to_pkg.name, &to_pkg.version);
                        (from_hash, to_hash)
                    } else {
                        (None, None)
                    };

                    updated_deps.push(CrateDiffInfo {
                        name: from_pkg.name,
                        from_version: Some(from_pkg.version),
                        from_hash,
                        to_version: Some(to_pkg.version),
                        to_hash,
                        repository,
                    });
                }
            } else {
                let from_hash = if !self.diff_rs {
                    self.registry_manager
                        .get_crate_hash(&from_pkg.name, &from_pkg.version)
                } else {
                    None
                };

                removed_deps.push(CrateDiffInfo {
                    name: from_pkg.name,
                    from_version: Some(from_pkg.version),
                    from_hash,
                    to_version: None,
                    to_hash: None,
                    repository,
                });
            }
        }

        // conver remaining to_nested_packages into the added changes
        let mut added_deps = vec![];
        for dep in to_nested_packages {
            let (to_hash, repository) = if !self.diff_rs {
                let to_hash = self
                    .registry_manager
                    .get_crate_hash(&dep.name, &dep.version);
                let repository = self
                    .registry_manager
                    .get_crate_repository(&dep.name, Some(&dep.version));
                (to_hash, repository)
            } else {
                (None, None)
            };

            added_deps.push(CrateDiffInfo {
                name: dep.name,
                from_version: None,
                from_hash: None,
                to_version: Some(dep.version),
                to_hash,
                repository,
            });
        }

        (removed_deps, added_deps, updated_deps)
    }

    fn get_nested_packages(
        &self,
        crate_name: &str,
        version: Option<&Version>,
    ) -> Vec<PackageIdInfo> {
        let Some(version) = version else {
            return vec![];
        };
        let from_registry_path = self.registry_manager.get_crate_path(crate_name, version);
        let cargo_meta = match CargoMeta::new(&from_registry_path) {
            Ok(cargo_meta) => cargo_meta,
            Err(err) => {
                eprintln!("Cannot get cargo metadata for '{crate_name}' crate. Error: {err}");
                return vec![];
            }
        };
        cargo_meta.workspace_nested_packages()
    }

    fn add_diff(
        &self,
        target_name: &str,
        crate_name: &str,
        from_version: Option<Version>,
        to_version: Option<Version>,
        repository: Option<String>,
        target_version_diffs: &mut HashMap<String, Vec<CrateDiffInfo>>,
    ) {
        if let Some(from_version) = &from_version
            && let Some(to_version) = &to_version
            && from_version == to_version
        {
            return;
        }

        let (from_hash, to_hash, repository) = if !self.diff_rs {
            let from_hash = from_version
                .as_ref()
                .and_then(|version| self.registry_manager.get_crate_hash(crate_name, version));
            let to_hash = to_version
                .as_ref()
                .and_then(|version| self.registry_manager.get_crate_hash(crate_name, version));
            (from_hash, to_hash, repository)
        } else {
            (None, None, None)
        };
        let deps = target_version_diffs.entry(target_name.into()).or_default();

        deps.push(CrateDiffInfo {
            name: crate_name.into(),
            from_version,
            from_hash,
            to_version,
            to_hash,
            repository, // TODO: can a repository of the same crate change between versions?
        });
    }
}
