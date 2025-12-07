use std::path::PathBuf;

use clap::Parser;
use semver::Version;

use crate::crate_diff_request::CrateDiffRequest;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    /// Path to `cargo` executable.  If not set, this will use the
    /// the `$CARGO` environment variable, and if that is not set, will
    /// simply be `cargo`
    #[arg(short, long)]
    pub cargo_path: Option<PathBuf>,
    /// Path to `Cargo.toml`
    #[arg(short, long, default_value = ".")]
    pub manifest_path: PathBuf,
    /// If true then show diffs for all nested dependencies otherwise only direct ones
    #[arg(short = 'a', long)]
    pub show_all: bool,
    /// Group changes per direct dependency
    #[arg(short, long)]
    pub group: bool,
    /// Show human readable output
    #[arg(short, long)]
    pub verbose: bool,
    /// List of crates with optional versions to inspect
    #[arg(value_parser = parse_crate_diff_info)]
    pub crates: Vec<CrateDiffRequest>,
}

fn parse_crate_diff_info(s: &str) -> Result<CrateDiffRequest, String> {
    let parts: Vec<_> = s.split('@').collect();

    if parts.len() == 1 {
        // just crate name: ddd serde
        return Ok(CrateDiffRequest {
            crate_name: s.into(),
            from_version: None,
            to_version: None,
        });
    }

    if parts.len() == 2 {
        // crate name and version definition: ddd serde@...
        let version_parts: Vec<_> = parts[1].split('-').collect();
        if version_parts.len() == 1 {
            // just single version or no version:
            //   ddd serde@1.0.223  # same as: ddd serde@-1.0.223
            //   ddd serde@         # same as: ddd serde
            let to_version = if version_parts[0].trim().is_empty() {
                None // ddd serde@
            } else {
                // ddd serde@1.0.223
                Some(Version::parse(version_parts[0]).map_err(|err| err.to_string())?)
            };

            return Ok(CrateDiffRequest {
                crate_name: parts[0].into(),
                from_version: None,
                to_version,
            });
        }

        if version_parts.len() == 2 {
            // from/to version range:
            //   ddd serde@1.0.223-1.0.226
            //   ddd serde@1.0.223-           # diff from 1.0.223 to the latest
            //   ddd serde@-1.0.223           # diff of the current crate serde version to 1.0.223
            //   ddd serde@-                  # same as: ddd serde
            let from_version = if version_parts[0].trim().is_empty() {
                None //   ddd serde@-...
            } else {
                //   ddd serde@1.0.223-...
                Some(Version::parse(version_parts[0]).map_err(|err| err.to_string())?)
            };
            let to_version = if version_parts[1].trim().is_empty() {
                None //   ddd serde@...-
            } else {
                //   ddd serde@...-1.0.223
                Some(Version::parse(version_parts[1]).map_err(|err| err.to_string())?)
            };

            return Ok(CrateDiffRequest {
                crate_name: parts[0].into(),
                from_version,
                to_version,
            });
        }
    }
    Err(format!("Wrong crate version format: {s}"))
}

#[cfg(test)]
mod tests {
    use semver::Version;

    use crate::cli::parse_crate_diff_info;

    #[test]
    fn test_simple_crate_definition() {
        check_simple_crate_def("serde", "serde");
        check_simple_crate_def("serde@", "serde");
        check_simple_crate_def("serde@-", "serde");
    }

    #[test]
    fn test_crate_with_target_version() {
        check_rate_with_target_version("serde@1.0.225", "serde", Version::new(1, 0, 225));
        check_rate_with_target_version("serde@-1.0.223", "serde", Version::new(1, 0, 223));
    }

    #[test]
    fn test_crate_with_source_version() {
        let info = parse_crate_diff_info("serde@1.0.224-").expect("Wrong crate version definition");
        assert_eq!("serde", info.crate_name);
        assert_eq!(Some(Version::new(1, 0, 224)), info.from_version);
        assert_eq!(None, info.to_version);
    }

    #[test]
    fn test_crate_with_both_versions() {
        let info =
            parse_crate_diff_info("serde@1.0.224-1.0.228").expect("Wrong crate version definition");
        assert_eq!("serde", info.crate_name);
        assert_eq!(Some(Version::new(1, 0, 224)), info.from_version);
        assert_eq!(Some(Version::new(1, 0, 228)), info.to_version);
    }

    #[test]
    fn test_multiple_at_fail() {
        let result = parse_crate_diff_info("serde@1.0.224@1.0.228");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_dash_fail() {
        let result = parse_crate_diff_info("serde@1.0.224-1.0.226-1.0.228");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_first_version_fail() {
        let result = parse_crate_diff_info("serde@1.0.22*");
        assert!(result.is_err());
    }

    #[test]
    fn test_wrong_second_version_fail() {
        let result = parse_crate_diff_info("serde@-1.0.22*");
        assert!(result.is_err());
    }

    fn check_simple_crate_def(s: &str, expected_name: &str) {
        let info = parse_crate_diff_info(s).expect("Wrong crate version definition");
        assert_eq!(expected_name, info.crate_name, "{s}");
        assert_eq!(None, info.from_version, "{s}");
        assert_eq!(None, info.to_version, "{s}");
    }

    fn check_rate_with_target_version(s: &str, expected_name: &str, expected_version: Version) {
        let info = parse_crate_diff_info(s).expect("Wrong crate version definition");
        assert_eq!(expected_name, info.crate_name, "{s}");
        assert_eq!(None, info.from_version, "{s}");
        assert_eq!(Some(expected_version), info.to_version, "{s}");
    }
}
