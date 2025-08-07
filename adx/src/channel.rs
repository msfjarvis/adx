use crate::version::Version;
use clap::ValueEnum;
use clap::builder::PossibleValue;
use semver::Prerelease;
use std::cmp::{Eq, PartialEq, PartialOrd};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::str::FromStr;
use thiserror::Error;

/// Release channels for androidx packages
/// Since we're deriving `PartialOrd` automatically, the order
/// of these fields is crucial. Sort by stability, not alphabetical
/// order.
#[derive(Debug, Eq, PartialEq, PartialOrd, Clone, Copy)]
pub(crate) enum Channel {
    Dev,
    Alpha,
    Beta,
    Rc,
    Stable,
}

impl ValueEnum for Channel {
    fn value_variants<'a>() -> &'a [Self] {
        &[Self::Dev, Self::Alpha, Self::Beta, Self::Rc, Self::Stable]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            Channel::Dev => Some(PossibleValue::new("dev").alias("d")),
            Channel::Alpha => Some(PossibleValue::new("alpha").alias("a")),
            Channel::Beta => Some(PossibleValue::new("beta").alias("b")),
            Channel::Stable => Some(PossibleValue::new("stable").alias("s")),
            Channel::Rc => Some(PossibleValue::new("rc").alias("r")),
        }
    }
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
            "alpha" | "a" => Ok(Channel::Alpha),
            "beta" | "b" => Ok(Channel::Beta),
            "dev" | "d" => Ok(Channel::Dev),
            "rc" | "r" => Ok(Channel::Rc),
            "stable" | "s" => Ok(Channel::Stable),
            _ => Err(ChannelError::NoMatchFound),
        }
    }
}

impl TryFrom<&Version> for Channel {
    type Error = ChannelError;

    fn try_from(value: &Version) -> Result<Self, Self::Error> {
        match value {
            Version::SemVer(version) => {
                if version.pre == Prerelease::EMPTY {
                    Ok(Channel::Stable)
                } else {
                    let pre_str = version.pre.to_string();
                    if pre_str.starts_with("alpha") {
                        Ok(Channel::Alpha)
                    } else if pre_str.starts_with("beta") {
                        Ok(Channel::Beta)
                    } else if pre_str.starts_with("dev") {
                        Ok(Channel::Dev)
                    } else if pre_str.starts_with("rc") {
                        Ok(Channel::Rc)
                    } else {
                        Err(ChannelError::FailedToParseVersion(value.clone()))
                    }
                }
            }
            Version::CalVer(_) => Ok(Channel::Stable),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Channel;
    use crate::version::Version;
    use semver::{BuildMetadata, Prerelease, Version as Semver};
    use std::convert::TryFrom;

    #[test]
    fn alpha_version() {
        let v = Version::SemVer(Semver {
            major: 1,
            minor: 1,
            patch: 0,
            pre: Prerelease::new("alpha01").unwrap(),
            build: BuildMetadata::EMPTY,
        });
        let channel = Channel::try_from(&v);
        assert_eq!(Channel::Alpha, channel.unwrap());
    }

    #[test]
    fn beta_version() {
        let v = Version::SemVer(Semver {
            major: 1,
            minor: 1,
            patch: 0,
            pre: Prerelease::new("beta01").unwrap(),
            build: BuildMetadata::EMPTY,
        });
        let channel = Channel::try_from(&v);
        assert_eq!(Channel::Beta, channel.unwrap());
    }

    #[test]
    fn dev_version() {
        let v = Version::SemVer(Semver {
            major: 1,
            minor: 1,
            patch: 0,
            pre: Prerelease::new("dev01").unwrap(),
            build: BuildMetadata::EMPTY,
        });
        let channel = Channel::try_from(&v);
        assert_eq!(Channel::Dev, channel.unwrap());
    }

    #[test]
    fn rc_version() {
        let v = Version::SemVer(Semver {
            major: 1,
            minor: 1,
            patch: 0,
            pre: Prerelease::new("rc01").unwrap(),
            build: BuildMetadata::EMPTY,
        });
        let channel = Channel::try_from(&v);
        assert_eq!(Channel::Rc, channel.unwrap());
    }

    #[test]
    fn stable_version() {
        let v = Version::SemVer(Semver::new(1, 1, 1));
        let channel = Channel::try_from(&v);
        assert_eq!(Channel::Stable, channel.unwrap());
    }

    #[test]
    fn calver() {
        let v = Version::CalVer((2023, 4, 0));
        let channel = Channel::try_from(&v);
        assert_eq!(Channel::Stable, channel.unwrap());
    }

    #[test]
    fn cmp_channels() {
        assert!(Channel::Stable > Channel::Rc);
        assert!(Channel::Rc > Channel::Beta);
        assert!(Channel::Beta > Channel::Alpha);
        assert!(Channel::Alpha > Channel::Dev);
    }

    #[test]
    fn cmp_parsed_versions() {
        let stable = Channel::try_from(&Version::SemVer(Semver::parse("1.1.0").unwrap())).unwrap();
        let rc = Channel::try_from(&Version::SemVer(Semver::parse("1.1.0-rc01").unwrap())).unwrap();
        let alpha =
            Channel::try_from(&Version::SemVer(Semver::parse("1.1.0-alpha01").unwrap())).unwrap();
        let beta =
            Channel::try_from(&Version::SemVer(Semver::parse("1.1.0-beta01").unwrap())).unwrap();
        assert!(Channel::Stable >= stable);
        assert!(Channel::Stable >= rc);
        assert!(Channel::Stable >= alpha);
        assert!(Channel::Stable >= beta);
        assert!(Channel::Rc >= rc);
        assert!(Channel::Rc >= beta);
        assert!(Channel::Rc >= alpha);
        assert!(Channel::Beta >= beta);
        assert!(Channel::Beta >= alpha);
        assert!(Channel::Alpha >= alpha);
    }
}
