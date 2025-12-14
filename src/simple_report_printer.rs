use std::collections::BTreeSet;

use crate::crate_diff_info::CrateDiffInfo;
use crate::dependency_diff::DependencyDiff;
use crate::diff_report::DiffReport;
use crate::field_size::get_dep_max_len;

pub struct SimpleReportPrinter {
    group: bool,
    diff_rs: bool,
    max_name_len: usize,
    max_from_ver_len: usize,
    max_to_ver_len: usize,
}

impl SimpleReportPrinter {
    pub fn new(group: bool, diff_rs: bool) -> Self {
        Self {
            group,
            diff_rs,
            max_name_len: 0,
            max_from_ver_len: 0,
            max_to_ver_len: 0,
        }
    }

    pub fn print(&mut self, report: &DiffReport) {
        let mut not_first = false;
        for (target_name, diffs) in &report.dependency_diffs {
            if not_first {
                println!();
            } else {
                not_first = true;
            }

            if !target_name.is_empty() {
                println!(": {target_name}");
            }

            for diff in diffs {
                get_dep_max_len(
                    diff,
                    &mut self.max_name_len,
                    &mut self.max_from_ver_len,
                    &mut self.max_to_ver_len,
                );
            }

            if self.group {
                for diff in diffs {
                    self.print_grouped_diff(diff);
                }
            } else {
                self.print_diffs(diffs);
            }
        }
    }

    fn print_grouped_diff(&self, diff: &DependencyDiff) {
        self.print_crate_diff(&diff.diff, "#");

        for dep in &diff.updated_deps {
            self.print_crate_diff(dep, "=");
        }

        for dep in &diff.added_deps {
            self.print_crate_diff(dep, "+");
        }

        for dep in &diff.removed_deps {
            self.print_crate_diff(dep, "-");
        }
    }

    fn print_diffs(&self, diffs: &[DependencyDiff]) {
        let mut updated_deps = BTreeSet::new();
        let mut added_deps = BTreeSet::new();
        let mut removed_deps = BTreeSet::new();

        // print direct dependencies first
        for diff in diffs {
            self.print_crate_diff(&diff.diff, "#");

            // consolidate nested dependencies
            diff.updated_deps.iter().for_each(|d| {
                updated_deps.insert(d.clone());
            });
            diff.added_deps.iter().for_each(|d| {
                added_deps.insert(d.clone());
            });
            diff.removed_deps.iter().for_each(|d| {
                removed_deps.insert(d.clone());
            });
        }

        // print nested dependencies
        for dep in updated_deps {
            self.print_crate_diff(&dep, "=");
        }

        for dep in added_deps {
            self.print_crate_diff(&dep, "+");
        }

        for dep in removed_deps {
            self.print_crate_diff(&dep, "-");
        }
    }

    fn print_crate_diff(&self, diff: &CrateDiffInfo, prefix: &str) {
        let from_version_str = diff
            .from_version
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();
        let to_version_str = diff
            .to_version
            .as_ref()
            .map(|v| v.to_string())
            .unwrap_or_default();

        print!(
            "{prefix} {:1$} {from_version_str:2$} {to_version_str:3$} ",
            diff.name, self.max_name_len, self.max_from_ver_len, self.max_to_ver_len
        );

        if diff.from_version.is_some() {
            if diff.to_version.is_some() {
                if self.diff_rs {
                    println!(
                        "https://diff.rs/{}/{from_version_str}/{to_version_str}",
                        diff.name
                    );
                } else if let Some(repository) = &diff.repository
                    && repository.starts_with("https://github.com/")
                    && let (Some(from_hash), Some(to_hash)) = (&diff.from_hash, &diff.to_hash)
                {
                    println!(
                        "{repository}/compare/{}...{}",
                        &from_hash[..7],
                        &to_hash[..7]
                    );
                } else {
                    let repository = if let Some(repository) = &diff.repository {
                        repository
                    } else {
                        "<unknown-repository>"
                    };
                    println!("{repository}");
                }
            } else {
                // removed dependency
                if !self.diff_rs
                    && let Some(repository) = &diff.repository
                    && repository.starts_with("https://github.com/")
                    && let Some(from_hash) = &diff.from_hash
                {
                    println!("{repository}/commit/{from_hash}");
                } else {
                    println!(
                        "https://diff.rs/{}/{from_version_str}/{from_version_str}",
                        diff.name
                    );
                }
            }
        } else if diff.to_version.is_some() {
            // added dependency
            if !self.diff_rs
                && let Some(repository) = &diff.repository
                && repository.starts_with("https://github.com/")
                && let Some(to_hash) = &diff.to_hash
            {
                println!("{repository}/commit/{to_hash}");
            } else {
                println!(
                    "https://diff.rs/{}/{to_version_str}/{to_version_str}",
                    diff.name
                );
            }
        } else {
            // unknown
            println!("{}", diff.repository.as_deref().unwrap_or("<unknown>"));
        }
    }
}
