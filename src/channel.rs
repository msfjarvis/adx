use semver::Version;
use std::cmp::{Eq, PartialEq, PartialOrd};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::str::FromStr;
use thiserror::Error;

/// Release channels for AndroidX packages
/// Since we're deriving [PartialOrd] automatically, the order
/// of these fields is crucial. Sort by stability, not alphabetical
/// order.
#[derive(Debug, Eq, PartialEq, PartialOrd)]
pub(crate) enum Channel {
    Dev,
    Alpha,
    Beta,
    RC,
    Stable,
}

#[derive(Debug, Error)]
pub(crate) enum ChannelError {
    #[error("no match found")]
    NoMatchFound,
    #[error("failed to determine channel for {0}")]
    FailedToParseVersion(Version),
}

impl FromStr for Channel {
    type Err = ChannelError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "alpha" => Ok(Channel::Alpha),
            "beta" => Ok(Channel::Beta),
            "dev" => Ok(Channel::Dev),
            "rc" => Ok(Channel::RC),
            "stable" => Ok(Channel::Stable),
            _ => Err(ChannelError::NoMatchFound),
        }
    }
}

impl TryFrom<Version> for Channel {
    type Error = ChannelError;

    fn try_from(value: Version) -> Result<Self, Self::Error> {
        if value.pre.is_empty() {
            return Ok(Channel::Stable);
        };
        if let Some(pre) = value.pre.get(0) {
            let pre_str = pre.to_string();
            return if pre_str.starts_with("alpha") {
                Ok(Channel::Alpha)
            } else if pre_str.starts_with("beta") {
                Ok(Channel::Beta)
            } else if pre_str.starts_with("dev") {
                Ok(Channel::Dev)
            } else if pre_str.starts_with("rc") {
                Ok(Channel::RC)
            } else {
                Err(ChannelError::FailedToParseVersion(value))
            };
        } else {
            return Ok(Channel::Stable);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Channel;
    use semver::{AlphaNumeric, Version};
    use std::convert::TryFrom;

    #[test]
    fn alpha_version() {
        let v = Version {
            major: 1,
            minor: 1,
            patch: 0,
            pre: vec![AlphaNumeric("alpha01".to_owned())],
            build: vec![],
        };
        let channel = Channel::try_from(v);
        assert_eq!(Channel::Alpha, channel.unwrap());
    }

    #[test]
    fn beta_version() {
        let v = Version {
            major: 1,
            minor: 1,
            patch: 0,
            pre: vec![AlphaNumeric("beta01".to_owned())],
            build: vec![],
        };
        let channel = Channel::try_from(v);
        assert_eq!(Channel::Beta, channel.unwrap());
    }

    #[test]
    fn dev_version() {
        let v = Version {
            major: 1,
            minor: 1,
            patch: 0,
            pre: vec![AlphaNumeric("dev01".to_owned())],
            build: vec![],
        };
        let channel = Channel::try_from(v);
        assert_eq!(Channel::Dev, channel.unwrap());
    }

    #[test]
    fn rc_version() {
        let v = Version {
            major: 1,
            minor: 1,
            patch: 0,
            pre: vec![AlphaNumeric("rc01".to_owned())],
            build: vec![],
        };
        let channel = Channel::try_from(v);
        assert_eq!(Channel::RC, channel.unwrap());
    }

    #[test]
    fn stable_version() {
        let v = Version::new(1, 1, 1);
        let channel = Channel::try_from(v);
        assert_eq!(Channel::Stable, channel.unwrap());
    }

    #[test]
    fn cmp_channels() {
        assert!(Channel::Stable > Channel::RC);
        assert!(Channel::RC > Channel::Beta);
        assert!(Channel::Beta > Channel::Alpha);
        assert!(Channel::Alpha > Channel::Dev);
    }
}
