//! The PostgreSQL oracles. Each oracle is a real PostgreSQL server built from a pin.
//!
//! Version 19 is the reference for rupg. Versions 14 to 18 are the oracles for the shims of `rupg.compat_version`. See `spec/05-compatibility.md` of tamnd/rupg.

/// One oracle build.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct Pin {
    /// The major version.
    pub(crate) major: u32,
    /// The release that the pin gives.
    pub(crate) release: &'static str,
    /// The git branch or tag in the PostgreSQL repository.
    pub(crate) git_ref: &'static str,
    /// The commit, for a pin on a branch. A pin on a tag has no commit here.
    pub(crate) commit: Option<&'static str>,
}

/// The oracle pins, oldest first. They agree with the vendored files of tamnd/rupg.
pub(crate) const PINS: [Pin; 6] = [
    Pin { major: 14, release: "14.24", git_ref: "REL_14_24", commit: None },
    Pin { major: 15, release: "15.19", git_ref: "REL_15_19", commit: None },
    Pin { major: 16, release: "16.15", git_ref: "REL_16_15", commit: None },
    Pin { major: 17, release: "17.11", git_ref: "REL_17_11", commit: None },
    Pin { major: 18, release: "18.6", git_ref: "REL_18_6", commit: None },
    Pin { major: 19, release: "19beta4", git_ref: "REL_19_STABLE", commit: Some("7d3d2db7") },
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_pin_for_each_major_version_from_14_to_19() {
        let majors: Vec<u32> = PINS.iter().map(|p| p.major).collect();
        assert_eq!(majors, [14, 15, 16, 17, 18, 19]);
    }

    #[test]
    fn each_release_starts_with_its_major_version() {
        for pin in PINS {
            assert!(pin.release.starts_with(&pin.major.to_string()));
        }
    }

    #[test]
    fn only_the_reference_is_on_a_branch() {
        let on_branch: Vec<u32> =
            PINS.iter().filter(|p| p.commit.is_some()).map(|p| p.major).collect();
        assert_eq!(on_branch, [19]);
    }
}
