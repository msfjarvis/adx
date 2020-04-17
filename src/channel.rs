use std::cmp::Eq;
use std::cmp::Ord;
use std::cmp::Ordering;
use std::cmp::PartialEq;
use std::cmp::PartialOrd;
use std::fmt::Display;
use std::fmt::Formatter;

#[repr(usize)]
#[derive(Copy, Eq, Debug, PartialEq, Hash)]
pub enum Channel {
    Dev = 0,
    Alpha,
    Beta,
    RC,
    Stable,
}

impl Channel {
    pub fn from_version(version: &str) -> Channel {
        if version.contains("dev") {
            Channel::Dev
        } else if version.contains("alpha") {
            Channel::Alpha
        } else if version.contains("beta") {
            Channel::Beta
        } else if version.contains("rc") {
            Channel::RC
        } else if !version.contains('-') {
            Channel::Stable
        } else {
            panic!("Failed to determine channel for {}", version)
        }
    }
}

impl Clone for Channel {
    #[inline]
    fn clone(&self) -> Channel {
        *self
    }
}

impl Display for Channel {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Channel::Dev => "dev",
                Channel::Alpha => "alpha",
                Channel::Beta => "beta",
                Channel::RC => "rc",
                Channel::Stable => "stable",
            }
        )
    }
}

impl Ord for Channel {
    #[inline]
    fn cmp(&self, other: &Channel) -> Ordering {
        (*self as usize).cmp(&(*other as usize))
    }
}

impl PartialOrd for Channel {
    #[inline]
    fn partial_cmp(&self, other: &Channel) -> Option<Ordering> {
        Some(self.cmp(other))
    }

    #[inline]
    fn lt(&self, other: &Channel) -> bool {
        (*self as usize) < *other as usize
    }

    #[inline]
    fn le(&self, other: &Channel) -> bool {
        *self as usize <= *other as usize
    }

    #[inline]
    fn gt(&self, other: &Channel) -> bool {
        *self as usize > *other as usize
    }

    #[inline]
    fn ge(&self, other: &Channel) -> bool {
        *self as usize >= *other as usize
    }
}

#[cfg(test)]
mod test {
    use super::Channel;

    #[test]
    fn valid_channels_parsed_correctly() {
        assert_eq!(Channel::from_version("0.1.0-dev02"), Channel::Dev);
        assert_eq!(Channel::from_version("1.0.0-alpha02"), Channel::Alpha);
        assert_eq!(Channel::from_version("1.0.0-beta02"), Channel::Beta);
        assert_eq!(Channel::from_version("1.0.0-rc01"), Channel::RC);
        assert_eq!(Channel::from_version("1.0.0"), Channel::Stable);
    }

    #[test]
    #[should_panic]
    fn invalid_version_throws() {
        Channel::from_version("2.0.0-hakuna_matata");
    }

    #[test]
    fn channel_ordered_correctly() {
        // We prefer to go with stable as the latest revision if available, then
        // cascade in the order Stable => RC => Beta => Alpha => Dev until a
        // suitable candidate is found. These tests help ensure that we can use
        // simple comparison statements for the purpose.
        assert!(Channel::Stable > Channel::RC);
        assert!(Channel::RC > Channel::Beta);
        assert!(Channel::Beta > Channel::Alpha);
        assert!(Channel::Alpha > Channel::Dev);
    }
}
