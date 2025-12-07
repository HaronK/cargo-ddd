use crate::crate_diff_info::CrateDiffInfo;
use crate::dependency_diff::DependencyDiff;

pub fn get_dep_max_len(
    diff: &DependencyDiff,
    max_name_len: &mut usize,
    max_from_ver_len: &mut usize,
    max_to_ver_len: &mut usize,
) {
    get_crate_max_len(&diff.diff, max_name_len, max_from_ver_len, max_to_ver_len);

    get_crates_max_len(
        &diff.updated_deps,
        max_name_len,
        max_from_ver_len,
        max_to_ver_len,
    );

    get_crates_max_len(
        &diff.added_deps,
        max_name_len,
        max_from_ver_len,
        max_to_ver_len,
    );

    get_crates_max_len(
        &diff.removed_deps,
        max_name_len,
        max_from_ver_len,
        max_to_ver_len,
    );
}

pub fn get_crates_max_len(
    diffs: &[CrateDiffInfo],
    max_name_len: &mut usize,
    max_from_ver_len: &mut usize,
    max_to_ver_len: &mut usize,
) {
    for diff in diffs {
        get_crate_max_len(diff, max_name_len, max_from_ver_len, max_to_ver_len);
    }
}

fn get_crate_max_len(
    diff: &CrateDiffInfo,
    max_name_len: &mut usize,
    max_from_ver_len: &mut usize,
    max_to_ver_len: &mut usize,
) {
    *max_name_len = std::cmp::max(*max_name_len, diff.name.len());

    let from_len = diff
        .from_version
        .as_ref()
        .map(|v| format!("{v}").len())
        .unwrap_or_default();
    *max_from_ver_len = std::cmp::max(*max_from_ver_len, from_len);

    let to_len = diff
        .to_version
        .as_ref()
        .map(|v| format!("{v}").len())
        .unwrap_or_default();
    *max_to_ver_len = std::cmp::max(*max_to_ver_len, to_len);
}
