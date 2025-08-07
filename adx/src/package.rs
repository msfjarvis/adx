use crate::{channel::Channel, version::Version};
use std::fmt::{Debug, Display, Formatter, Result};

/// Struct that represents a Maven package
#[derive(Debug)]
pub(crate) struct MavenPackage {
    pub(crate) group_id: String,
    pub(crate) artifact_id: String,
    pub(crate) versions: Vec<Version>,
}

pub(crate) struct LatestPackage {
    pub(crate) group_id: String,
    pub(crate) artifact_id: String,
    pub(crate) version: Version,
}

impl MavenPackage {
    pub fn latest(&self, channel: Channel) -> Option<LatestPackage> {
        let mut versions: Vec<&Version> = self
            .versions
            .iter()
            .filter(|v| {
                if let Ok(c) = Channel::try_from(*v) {
                    c >= channel
                } else {
                    false
                }
            })
            .collect();
        versions.sort_by(|a, b| b.partial_cmp(a).unwrap());
        let version = if versions.is_empty() {
            return None;
        } else {
            (*versions.first().unwrap()).clone()
        };
        Some(LatestPackage {
            version,
            group_id: self.group_id.clone(),
            artifact_id: self.artifact_id.clone(),
        })
    }
}

impl Display for LatestPackage {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        write!(f, "{}:{}:{}", self.group_id, self.artifact_id, self.version)
    }
}
