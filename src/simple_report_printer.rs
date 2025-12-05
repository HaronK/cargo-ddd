use crate::crate_diff_info::CrateDiffInfo;
use crate::dependency_diff::DependencyDiff;
use crate::diff_report::DiffReport;
use crate::field_size::get_dep_max_len;

pub fn print_simple(report: &DiffReport) {
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

        let mut max_name_len = 0;
        let mut max_from_ver_len = 0;
        let mut max_to_ver_len = 0;

        for diff in diffs {
            get_dep_max_len(
                diff,
                &mut max_name_len,
                &mut max_from_ver_len,
                &mut max_to_ver_len,
            );
        }

        for diff in diffs {
            print_dep_diff(diff, max_name_len, max_from_ver_len, max_to_ver_len);
        }
    }
}

fn print_dep_diff(
    diff: &DependencyDiff,
    max_name_len: usize,
    max_from_ver_len: usize,
    max_to_ver_len: usize,
) {
    print_crate_diff(
        &diff.diff,
        "#",
        max_name_len,
        max_from_ver_len,
        max_to_ver_len,
    );

    if diff.removed_deps.is_empty() && diff.added_deps.is_empty() && diff.updated_deps.is_empty() {
        return;
    }

    if !diff.updated_deps.is_empty() {
        for dep in &diff.updated_deps {
            print_crate_diff(dep, "=", max_name_len, max_from_ver_len, max_to_ver_len);
        }
    }

    if !diff.added_deps.is_empty() {
        for dep in &diff.added_deps {
            print_crate_diff(dep, "+", max_name_len, max_from_ver_len, max_to_ver_len);
        }
    }

    if !diff.removed_deps.is_empty() {
        for dep in &diff.removed_deps {
            print_crate_diff(dep, "-", max_name_len, max_from_ver_len, max_to_ver_len);
        }
    }
}

fn print_crate_diff(
    diff: &CrateDiffInfo,
    prefix: &str,
    max_name_len: usize,
    max_from_ver_len: usize,
    max_to_ver_len: usize,
) {
    let from_version_str = diff
        .from_version
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or("<undefined>".into());
    let to_version_str = diff
        .to_version
        .as_ref()
        .map(|v| v.to_string())
        .unwrap_or("<undefined>".into());

    print!(
        "{prefix} {:1$} {from_version_str:2$} {to_version_str:3$} ",
        diff.name, max_name_len, max_from_ver_len, max_to_ver_len
    );

    if diff.from_version.is_some() {
        if diff.to_version.is_some() {
            if let Some(repository) = &diff.repository
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
                    "<undefined>"
                };
                println!("{repository}");
            }
        } else {
            // removed dependency
            if let Some(repository) = &diff.repository
                && repository.starts_with("https://github.com/")
                && let Some(from_hash) = &diff.from_hash
            {
                println!("{repository}/commit/{from_hash}");
            } else {
                println!(
                    "{} {}",
                    diff.repository.as_deref().unwrap_or("<undefined>"),
                    diff.from_hash.as_deref().unwrap_or("<undefined>"),
                );
            }
        }
    } else if diff.to_version.is_some() {
        // added dependency
        if let Some(repository) = &diff.repository
            && repository.starts_with("https://github.com/")
            && let Some(to_hash) = &diff.to_hash
        {
            println!("{repository}/commit/{to_hash}");
        } else {
            println!(
                "{} {}",
                diff.repository.as_deref().unwrap_or("<undefined>"),
                diff.to_hash.as_deref().unwrap_or("<undefined>"),
            );
        }
    } else {
        // unknown
        println!("{}", diff.repository.as_deref().unwrap_or("<unknown>"));
    }
}
