use semver::Version;

/// The crate diff information
#[derive(Debug)]
pub struct CrateDiffInfo {
    /// Crate name
    pub name: String,
    /// Initial crate version
    pub from_version: Option<Version>,
    /// Initial crate repository commit hash
    pub from_hash: Option<String>,
    /// Target crate version
    pub to_version: Option<Version>,
    /// Target crate repository commit hash
    pub to_hash: Option<String>,
    /// Crate repository path
    pub repository: Option<String>,
}

impl CrateDiffInfo {
    pub fn print(&self, indent: usize) {
        let ident_str = " ".repeat(indent * 2);

        if let Some(from_version) = &self.from_version {
            if let Some(to_version) = &self.to_version {
                // updated dependency
                let version_change = if from_version < to_version {
                    "upgraded"
                } else {
                    "downgraded"
                };

                println!("{ident_str}{}: {version_change}", self.name);
                if let Some(repository) = &self.repository
                    && repository.starts_with("https://github.com/")
                    && let (Some(from_hash), Some(to_hash)) = (&self.from_hash, &self.to_hash)
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
                        &self.from_hash.as_deref().unwrap_or("<unknown-commit-hash>"),
                    );
                    println!(
                        "{ident_str}  To:   {to_version} {}",
                        &self.to_hash.as_deref().unwrap_or("<unknown-commit-hash>"),
                    );

                    let repository = if let Some(repository) = &self.repository {
                        repository
                    } else {
                        "<unknown>"
                    };
                    println!("{ident_str}  Repository: {repository}");
                }
            } else {
                // removed dependency
                if let Some(repository) = &self.repository
                    && repository.starts_with("https://github.com/")
                    && let Some(from_hash) = &self.from_hash
                {
                    println!(
                        "{ident_str}{}: {from_version} {repository}/commit/{from_hash}",
                        self.name
                    );
                } else {
                    println!(
                        "{ident_str}{}: {from_version} Repository: {} Commit: {}",
                        self.name,
                        self.repository.as_deref().unwrap_or("<unknown>"),
                        self.from_hash.as_deref().unwrap_or("<unknown>"),
                    );
                }
            }
        } else if let Some(to_version) = &self.to_version {
            // added dependency
            if let Some(repository) = &self.repository
                && repository.starts_with("https://github.com/")
                && let Some(to_hash) = &self.to_hash
            {
                println!(
                    "{ident_str}{}: {to_version} {repository}/commit/{to_hash}",
                    self.name
                );
            } else {
                println!(
                    "{ident_str}{}: {to_version} Repository: {} Commit: {}",
                    self.name,
                    self.repository.as_deref().unwrap_or("<unknown>"),
                    self.to_hash.as_deref().unwrap_or("<unknown>"),
                );
            }
        } else {
            // unknown
            println!(
                "{ident_str}{}: Repository: {}",
                self.name,
                self.repository.as_deref().unwrap_or("<unknown>")
            );
        }
    }
}
