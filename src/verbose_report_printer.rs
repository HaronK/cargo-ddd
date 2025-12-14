use std::collections::BTreeSet;

use crate::crate_diff_info::CrateDiffInfo;
use crate::dependency_diff::DependencyDiff;
use crate::diff_report::DiffReport;

pub struct VerboseReportPrinter {
    group: bool,
    diff_rs: bool,
}

impl VerboseReportPrinter {
    pub fn new(group: bool, diff_rs: bool) -> Self {
        Self { group, diff_rs }
    }

    pub fn print(&self, report: &DiffReport) {
        for (target_name, diffs) in &report.dependency_diffs {
            if target_name.is_empty() {
                println!("Default dependencies:");
            } else {
                println!("{target_name} dependencies:");
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
        self.print_crate_diff(&diff.diff, 1);

        if diff.removed_deps.is_empty()
            && diff.added_deps.is_empty()
            && diff.updated_deps.is_empty()
        {
            return;
        }

        println!("    Nested dependency diffs:");

        self.print_crates_diff(&diff.updated_deps, 3, "Updated");
        self.print_crates_diff(&diff.added_deps, 3, "Added");
        self.print_crates_diff(&diff.removed_deps, 3, "Removed");
    }

    fn print_diffs(&self, diffs: &[DependencyDiff]) {
        let mut updated_deps = BTreeSet::new();
        let mut added_deps = BTreeSet::new();
        let mut removed_deps = BTreeSet::new();

        // print direct dependencies first
        for diff in diffs {
            self.print_crate_diff(&diff.diff, 1);

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

        // TODO: get rid of this conversion
        let updated_deps: Vec<_> = updated_deps.into_iter().collect();
        let added_deps: Vec<_> = added_deps.into_iter().collect();
        let removed_deps: Vec<_> = removed_deps.into_iter().collect();

        // print nested dependencies
        self.print_crates_diff(&updated_deps, 0, "Updated");
        self.print_crates_diff(&added_deps, 0, "Added");
        self.print_crates_diff(&removed_deps, 0, "Removed");
    }

    fn print_crates_diff(&self, diffs: &[CrateDiffInfo], indent: usize, name: &str) {
        if !diffs.is_empty() {
            println!("{}{name}:", " ".repeat(indent * 2));
            for dep in diffs {
                self.print_crate_diff(dep, indent + 1);
            }
        }
    }

    fn print_crate_diff(&self, diff: &CrateDiffInfo, indent: usize) {
        let ident_str = " ".repeat(indent * 2);
        print!("{ident_str}{}:", diff.name);

        if let Some(from_version) = &diff.from_version {
            if let Some(to_version) = &diff.to_version {
                // updated dependency
                let version_change = if from_version < to_version {
                    "upgraded"
                } else {
                    "downgraded"
                };

                println!("{version_change}");
                if let Some(repository) = &diff.repository
                    && repository.starts_with("https://github.com/")
                    && let (Some(from_hash), Some(to_hash)) = (&diff.from_hash, &diff.to_hash)
                {
                    println!("{ident_str}  From: {from_version} {repository}/commit/{from_hash}");
                    println!("{ident_str}  To:   {to_version} {repository}/commit/{to_hash}");
                    println!(
                        "{ident_str}  Diff: {repository}/compare/{}...{}",
                        &from_hash[..7],
                        &to_hash[..7]
                    );
                } else {
                    println!(
                        "{ident_str}  From: {from_version} {}",
                        diff.from_hash.as_deref().unwrap_or("<unknown-commit>"),
                    );
                    println!(
                        "{ident_str}  To:   {to_version} {}",
                        diff.to_hash.as_deref().unwrap_or("<unknown-commit>"),
                    );

                    println!(
                        "{ident_str}  Repo: {}",
                        diff.repository.as_deref().unwrap_or("<unknown-repository>")
                    );
                }
                if self.diff_rs {
                    println!(
                        "{ident_str}  Diff: https://diff.rs/{}/{from_version}/{to_version}",
                        diff.name,
                    );
                }
            } else {
                // removed dependency
                println!(
                    "\n{ident_str}  From: {from_version} {}",
                    diff.from_hash.as_deref().unwrap_or("<unknown-commit>"),
                );
                print!("{ident_str}  Repo: ");

                if let Some(repository) = &diff.repository
                    && repository.starts_with("https://github.com/")
                    && let Some(from_hash) = &diff.from_hash
                {
                    println!("{repository}/commit/{from_hash}");
                } else {
                    println!(
                        "{} {}",
                        diff.repository.as_deref().unwrap_or("<unknown-repository>"),
                        diff.from_hash.as_deref().unwrap_or("<unknown-commit>"),
                    );
                }
                if self.diff_rs {
                    println!(
                        "{ident_str}  Diff: https://diff.rs/{}/{from_version}/{from_version}",
                        diff.name
                    );
                }
            }
        } else if let Some(to_version) = &diff.to_version {
            // added dependency
            println!(
                "\n{ident_str}  To:   {to_version} {}",
                diff.to_hash.as_deref().unwrap_or("<unknown-commit>"),
            );
            print!("{ident_str}  Repo: ");

            if let Some(repository) = &diff.repository
                && repository.starts_with("https://github.com/")
                && let Some(to_hash) = &diff.to_hash
            {
                println!("{repository}/commit/{to_hash}");
            } else {
                println!(
                    "{} {}",
                    diff.repository.as_deref().unwrap_or("<unknown-repository>"),
                    diff.to_hash.as_deref().unwrap_or("<unknown-commit>"),
                );
            }
            if self.diff_rs {
                println!(
                    "{ident_str}  Diff: https://diff.rs/{}/{to_version}/{to_version}",
                    diff.name
                );
            }
        } else {
            // unknown
            println!(
                "\n{ident_str}  Repo: {}",
                diff.repository.as_deref().unwrap_or("<unknown-repository>")
            );
        }
    }
}
