use std::collections::HashMap;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use cargo_metadata::{Metadata, MetadataCommand};

use crate::package_id_info::PackageIdInfo;
use crate::package_source::PackageSource;

/// Provides information from the crate metadata
pub struct CargoMeta {
    metadata: Metadata,
}

impl CargoMeta {
    pub fn new(registry_path: &Path) -> Result<Self> {
        let manifest_path = if registry_path.ends_with("Cargo.toml") {
            PathBuf::from(registry_path)
        } else {
            registry_path.join("Cargo.toml")
        };
        let metadata = MetadataCommand::new()
            .manifest_path(manifest_path)
            .exec()
            .context("Cannot get metadata for manifest: {manifest_path}")?;
        Ok(Self { metadata })
    }

    /// Try to extract registry path from the metadata
    pub fn registry_path(&self) -> Option<PathBuf> {
        self.metadata.packages.iter().find_map(|pkg| {
            let registry_path = pkg.manifest_path.to_string();
            if registry_path.contains(".cargo/") {
                PathBuf::from(registry_path)
                    .parent()
                    .and_then(|p| p.parent().map(|p| p.to_path_buf()))
                    .filter(|p| {
                        p.to_str()
                            .map(|s| s.contains(".cargo/registry/src/index.crates.io-"))
                            == Some(true)
                    })
            } else {
                None
            }
        })
    }

    /// Returns all entries of the crate in all workspace targets
    pub fn get_dependency_info(&self, crate_name: &str) -> HashMap<String, PackageIdInfo> {
        let mut dep_info = HashMap::new();
        for (target_name, deps) in self.workspace_member_dependencies() {
            // TODO: address dependency renaming
            if let Some(dep) = deps.into_iter().find(|d| d.name == crate_name) {
                dep_info.insert(target_name, dep);
            }
        }
        dep_info
    }

    /// Returns current crate direct dependencies of all workspace targets
    pub fn workspace_member_dependencies(&self) -> HashMap<String, Vec<PackageIdInfo>> {
        if let Some(resolve) = &self.metadata.resolve {
            let workspace_members = self.metadata.workspace_members.clone();
            let mut packages = HashMap::new();
            for node in &resolve.nodes {
                if workspace_members.contains(&node.id) {
                    let Some(info) = PackageIdInfo::from_package_id(&node.id) else {
                        continue;
                    };

                    let mut deps = vec![];
                    for dep in &node.dependencies {
                        // choose only non-local dependencies
                        if let Some(pkg_dep) = PackageIdInfo::from_package_id(dep)
                            && pkg_dep.source != PackageSource::Path
                        {
                            deps.push(pkg_dep);
                        }
                    }
                    packages.insert(info.name, deps);
                }
            }
            packages
        } else {
            eprintln!(
                "Metadata is not resolved for: {}",
                self.metadata.workspace_root
            );
            HashMap::default()
        }
    }

    /// Returns all nested dependencies.
    pub fn workspace_nested_packages(&self) -> Vec<PackageIdInfo> {
        if let Some(resolve) = &self.metadata.resolve {
            let workspace_members = self.metadata.workspace_members.clone();

            // collect dependencies of the workspace packages
            let mut workspace_dependencies = HashMap::new();
            for node in &resolve.nodes {
                if workspace_members.contains(&node.id) {
                    let mut deps = vec![];
                    for pkg_id in &node.dependencies {
                        let Some(pkg_dep) = PackageIdInfo::parse_source(pkg_id) else {
                            continue;
                        };
                        // choose only non-local dependencies
                        if pkg_dep != PackageSource::Path {
                            deps.push(pkg_id.clone());
                        }
                    }
                    workspace_dependencies.insert(node.id.clone(), deps);
                }
            }

            let mut packages = vec![];
            'next_node: for node in &resolve.nodes {
                // exclude workspace members and their direct dependencies
                for (target, deps) in &workspace_dependencies {
                    if *target == node.id {
                        continue 'next_node;
                    }
                    if deps.contains(&node.id) {
                        continue 'next_node;
                    }
                }
                if let Some(pkg_info) = PackageIdInfo::from_package_id(&node.id) {
                    packages.push(pkg_info)
                }
            }
            packages
        } else {
            eprintln!(
                "Metadata is not resolved for: {}",
                self.metadata.workspace_root
            );
            vec![]
        }
    }
}
