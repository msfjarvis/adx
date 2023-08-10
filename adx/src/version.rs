use semver::Version as Semver;
use std::fmt::Display;

#[derive(Clone, Debug, Eq, PartialOrd, Ord)]
pub enum Version {
    SemVer(Semver),
    CalVer((u16, u16, u16)),
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Version::SemVer(ver) => ver.fmt(f),
            Version::CalVer((year, month, day)) => {
                f.write_fmt(format_args!("{year:04}.{month:02}.{day:02}"))
            }
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::SemVer(l0), Self::SemVer(r0)) => l0 == r0,
            (Self::CalVer(l0), Self::CalVer(r0)) => l0 == r0,
            _ => false,
        }
    }
}
