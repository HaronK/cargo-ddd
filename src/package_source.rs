/// Crate source from the cargo metadata PackageId
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum PackageSource {
    /// Crate from the registry
    Registry,
    /// Local crate, i.e. workspace member
    Path,
    /// Crate from the Git repository
    Git,
    /// Unsupported source
    Unsupported(String),
}

impl std::fmt::Display for PackageSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let source = self.as_str();
        write!(f, "{source}")
    }
}

impl PackageSource {
    pub fn as_str(&self) -> &str {
        match self {
            Self::Registry => "registry",
            Self::Path => "path",
            Self::Git => "git",
            Self::Unsupported(source) => source,
        }
    }
}

impl From<&str> for PackageSource {
    fn from(value: &str) -> Self {
        match value {
            "registry" => Self::Registry,
            "path" => Self::Path,
            "git" => Self::Git,
            _ => Self::Unsupported(value.into()),
        }
    }
}
