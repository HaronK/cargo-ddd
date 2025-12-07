mod cargo_meta;
mod cargo_runner;
mod cli;
mod crate_diff_builder;
mod crate_diff_info;
mod crate_diff_request;
mod dependency_diff;
mod diff_report;
mod field_size;
mod package_id_info;
mod package_source;
mod registry_manager;
mod simple_report_printer;
mod verbose_report_printer;

use std::cmp::Ordering;
use std::collections::HashMap;

use anyhow::{Result, anyhow};
use clap::Parser;

use crate::cargo_meta::CargoMeta;
use crate::cli::Cli;
use crate::crate_diff_builder::CrateDiffBuilder;
use crate::crate_diff_info::CrateDiffInfo;
use crate::dependency_diff::DependencyDiff;
use crate::diff_report::DiffReport;
use crate::registry_manager::RegistryManager;
use crate::simple_report_printer::print_simple;
use crate::verbose_report_printer::print_verbose;

fn main() -> Result<()> {
    let cli = Cli::parse();

    let need_local_metadata =
        cli.crates.is_empty() || cli.crates.iter().any(|c| c.from_version.is_none());
    let cargo_meta = if need_local_metadata {
        Some(CargoMeta::new(&cli.manifest_path)?)
    } else {
        None
    };

    let registry_path = cargo_meta.as_ref().and_then(|cm| cm.registry_path());
    let registry_manager = RegistryManager::new(registry_path)?;
    let diff_builder = CrateDiffBuilder::new(registry_manager);

    let target_version_diffs = if cli.crates.is_empty() {
        // if no crates are provided in cli, use local crate dependencies that need an update
        let Some(cargo_meta) = cargo_meta else {
            return Err(anyhow!(
                "Run from the crate directory or provide path to the crate if no explicit crates are specified."
            ));
        };

        diff_builder.build_from_crate(&cargo_meta)
    } else if need_local_metadata {
        // crates are provided explicitly but we still need local crate (Cargo.toml)
        let Some(cargo_meta) = cargo_meta else {
            return Err(anyhow!(
                "One or more requested crates are required local crate. Run from the crate directory or provide path to the crate."
            ));
        };

        diff_builder.build_from_crate_deps(&cli.crates, &cargo_meta)
    } else {
        diff_builder.build_from_crates(&cli.crates)
    };

    if target_version_diffs.is_empty() {
        println!("All crates are up to date.");
        return Ok(());
    }

    // generate diff links
    let dependency_diffs: HashMap<_, _> = target_version_diffs
        .into_iter()
        .map(|(target_name, diffs)| {
            let dep_diff: Vec<_> = diffs
                .into_iter()
                .map(|diff| {
                    let (mut removed_deps, mut added_deps, mut updated_deps) = if cli.show_all {
                        diff_builder.build_nested_deps(&diff)
                    } else {
                        Default::default()
                    };

                    removed_deps.sort_by(compare_diffs);
                    added_deps.sort_by(compare_diffs);
                    updated_deps.sort_by(compare_diffs);

                    DependencyDiff {
                        diff,
                        removed_deps,
                        added_deps,
                        updated_deps,
                    }
                })
                .collect();
            (target_name, dep_diff)
        })
        .collect();

    let diff_report = DiffReport { dependency_diffs };

    if cli.verbose {
        print_verbose(&diff_report, cli.group);
    } else {
        print_simple(&diff_report, cli.group);
    }

    Ok(())
}

fn compare_diffs(a: &CrateDiffInfo, b: &CrateDiffInfo) -> Ordering {
    a.name
        .cmp(&b.name)
        .then_with(|| a.from_version.cmp(&b.from_version))
        .then_with(|| a.to_version.cmp(&b.to_version))
}
