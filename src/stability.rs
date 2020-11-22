use semver::Version;

/// This trait implements stability checks for [`semver::Version`](`semver::Version`)
pub(crate) trait Stability {
    fn is_alpha(&self) -> bool;
    fn is_beta(&self) -> bool;
    fn is_dev(&self) -> bool;
    fn is_rc(&self) -> bool;
    fn is_stable(&self) -> bool;
}

impl Stability for Version {
    fn is_alpha(&self) -> bool {
        if self.pre.is_empty() {
            return false;
        }
        if let Some(pre) = self.pre.get(0) {
            pre.to_string().starts_with("alpha")
        } else {
            false
        }
    }

    fn is_beta(&self) -> bool {
        if self.pre.is_empty() {
            return false;
        }
        if let Some(pre) = self.pre.get(0) {
            pre.to_string().starts_with("beta")
        } else {
            false
        }
    }

    fn is_dev(&self) -> bool {
        if self.pre.is_empty() {
            return false;
        }
        if let Some(pre) = self.pre.get(0) {
            pre.to_string().starts_with("dev")
        } else {
            false
        }
    }

    fn is_rc(&self) -> bool {
        if self.pre.is_empty() {
            return false;
        }
        if let Some(pre) = self.pre.get(0) {
            pre.to_string().starts_with("rc")
        } else {
            false
        }
    }

    fn is_stable(&self) -> bool {
        self.pre.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::Stability;
    use semver::{AlphaNumeric, Version};

    #[test]
    fn alpha_version() {
        let v = Version {
            major: 1,
            minor: 1,
            patch: 0,
            pre: vec![AlphaNumeric("alpha01".to_owned())],
            build: vec![],
        };
        assert!(v.is_alpha());
        assert!(!v.is_beta());
        assert!(!v.is_dev());
        assert!(v.is_prerelease());
        assert!(!v.is_rc());
        assert!(!v.is_stable());
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
        assert!(!v.is_alpha());
        assert!(v.is_beta());
        assert!(!v.is_dev());
        assert!(v.is_prerelease());
        assert!(!v.is_rc());
        assert!(!v.is_stable());
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
        assert!(!v.is_alpha());
        assert!(!v.is_beta());
        assert!(v.is_dev());
        assert!(v.is_prerelease());
        assert!(!v.is_rc());
        assert!(!v.is_stable());
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
        assert!(!v.is_alpha());
        assert!(!v.is_beta());
        assert!(!v.is_dev());
        assert!(v.is_prerelease());
        assert!(v.is_rc());
        assert!(!v.is_stable());
    }

    #[test]
    fn stable_version() {
        let v = Version::new(1, 1, 1);
        assert!(!v.is_alpha());
        assert!(!v.is_beta());
        assert!(!v.is_dev());
        assert!(!v.is_prerelease());
        assert!(!v.is_rc());
        assert!(v.is_stable());
    }
}
