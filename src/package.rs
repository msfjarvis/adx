use semver::Version;
use std::fmt::{Debug, Display, Formatter, Result};

/// Struct that represents a Maven package
#[derive(Debug)]
pub(crate) struct MavenPackage {
    pub(crate) group_id: String,
    pub(crate) artifact_id: String,
    pub(crate) latest_version: String,
    pub(crate) all_versions: Vec<Version>,
}

impl Display for MavenPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(
            f,
            "{}:{}:{}",
            self.group_id, self.artifact_id, self.latest_version,
        )
    }
}
