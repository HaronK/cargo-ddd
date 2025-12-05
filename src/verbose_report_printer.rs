use crate::crate_diff_info::CrateDiffInfo;
use crate::dependency_diff::DependencyDiff;
use crate::diff_report::DiffReport;
use crate::field_size::get_crates_max_len;

pub fn print_verbose(report: &DiffReport) {
    for (target_name, diffs) in &report.dependency_diffs {
        if target_name.is_empty() {
            println!("Default dependencies:");
        } else {
            println!("{target_name} dependencies:");
        }

        for diff in diffs {
            print_dep_diff(diff);
        }
    }
}

fn print_dep_diff(diff: &DependencyDiff) {
    print_crate_diff(&diff.diff, 1, 0, 0, 0);

    if diff.removed_deps.is_empty() && diff.added_deps.is_empty() && diff.updated_deps.is_empty() {
        return;
    }

    println!("    Nested dependency diffs:");

    print_crates_diff(&diff.removed_deps, "Removed");
    print_crates_diff(&diff.added_deps, "Added");
    print_crates_diff(&diff.updated_deps, "Updated");
}

fn print_crates_diff(diffs: &[CrateDiffInfo], name: &str) {
    if !diffs.is_empty() {
        let mut max_name_len = 0;
        let mut max_from_ver_len = 0;
        let mut max_to_ver_len = 0;

        get_crates_max_len(
            diffs,
            &mut max_name_len,
            &mut max_from_ver_len,
            &mut max_to_ver_len,
        );

        println!("      {name}:");
        for dep in diffs {
            print_crate_diff(dep, 4, max_name_len, max_from_ver_len, max_to_ver_len);
        }
    }
}

fn print_crate_diff(
    diff: &CrateDiffInfo,
    indent: usize,
    max_name_len: usize,
    max_from_ver_len: usize,
    max_to_ver_len: usize,
) {
    let ident_str = " ".repeat(indent * 2);
    print!("{ident_str}");

    if let Some(from_version) = &diff.from_version {
        if let Some(to_version) = &diff.to_version {
            // updated dependency
            let version_change = if from_version < to_version {
                "upgraded"
            } else {
                "downgraded"
            };

            println!("{}: {version_change}", diff.name);
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
        } else {
            // removed dependency
            print!(
                "{:1$} {from_version:2$} ",
                diff.name, max_name_len, max_from_ver_len
            );
            if let Some(repository) = &diff.repository
                && repository.starts_with("https://github.com/")
                && let Some(from_hash) = &diff.from_hash
            {
                println!("{repository}/commit/{from_hash}",);
            } else {
                println!(
                    "{} {}",
                    diff.repository.as_deref().unwrap_or("<unknown-repository>"),
                    diff.from_hash.as_deref().unwrap_or("<unknown-commit>"),
                );
            }
        }
    } else if let Some(to_version) = &diff.to_version {
        // added dependency
        print!(
            "{:1$} {to_version:2$} ",
            diff.name, max_name_len, max_to_ver_len
        );

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
    } else {
        // unknown
        println!(
            "{:1$} {2}",
            diff.name,
            max_name_len,
            diff.repository.as_deref().unwrap_or("<unknown-repository>")
        );
    }
}
