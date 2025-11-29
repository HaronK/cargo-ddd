use std::{ffi::OsStr, path::PathBuf, process::Command, str::from_utf8};

use anyhow::{Context, Result, anyhow};

/// Wrapper for cargo subcommands
pub struct CargoRunner {
    cargo_path: PathBuf,
}

impl CargoRunner {
    pub fn new(cargo_path: Option<PathBuf>) -> Self {
        let cargo_path = cargo_path
            .clone()
            .or_else(|| std::env::var("CARGO").map(PathBuf::from).ok())
            .unwrap_or_else(|| PathBuf::from("cargo"));

        Self { cargo_path }
    }

    /// Runs 'cargo <subcommand> <args>' command and returns its output
    pub fn run<I, S>(&self, subcommand: &str, args: I) -> Result<String>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<OsStr>,
    {
        let mut cmd = Command::new(&self.cargo_path);
        cmd.arg(subcommand);
        cmd.args(args);

        let output = cmd.output()?;
        if !output.status.success() {
            return Err(anyhow!(
                "Cannot run command: {cmd:?}\n{}",
                String::from_utf8(output.stderr)?
            ));
        }

        from_utf8(&output.stdout)
            .map(|o| o.to_string())
            .context("Cannot convert cargo command output")
    }
}
