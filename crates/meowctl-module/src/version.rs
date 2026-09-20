//! Comparing versions the way the lock files spell them.
//!
//! `v0.1.0` uses `golang.org/x/mod/semver`, which requires a leading `v` and
//! treats anything else as invalid. The lock files and manifests in the wild
//! write both: `deps.mod` has `version = "0.2.17"` and the registry index has
//! tags like `v0.2.17`. So a version is parsed leniently and compared
//! strictly.

use std::cmp::Ordering;
use std::fmt;
use std::hash::Hash as _;

/// A module version, or the absence of a requirement.
///
/// `none` is a real value in Minimal Version Selection: it orders below every
/// version and means "no requirement", which is how a module drops out of a
/// build list; see [R-MODULE-002].
#[derive(Debug, Clone)]
pub enum Version {
    /// No requirement.
    None,
    /// A semantic version, with the text it was written as.
    Semver {
        /// The parsed version, for comparison.
        parsed: semver::Version,
        /// What was written, so a lock file round-trips.
        written: String,
    },
}

impl Version {
    /// Parses a version, accepting a leading `v`.
    ///
    /// # Errors
    ///
    /// The text, when it is not `none` and not a semantic version.
    pub fn parse(text: &str) -> Result<Version, String> {
        if text == "none" || text.is_empty() {
            return Ok(Version::None);
        }
        let bare = text.strip_prefix('v').unwrap_or(text);
        semver::Version::parse(bare)
            .map(|parsed| Version::Semver {
                parsed,
                written: text.to_owned(),
            })
            .map_err(|e| format!("{text} is not a version: {e}"))
    }

    /// The text this was written as.
    #[must_use]
    pub fn as_str(&self) -> &str {
        match self {
            Version::None => "none",
            Version::Semver { written, .. } => written,
        }
    }

    /// Whether this is the absence of a requirement.
    #[must_use]
    pub const fn is_none(&self) -> bool {
        matches!(self, Version::None)
    }

    /// The greater of two versions.
    ///
    /// What Minimal Version Selection does at every edge: the selected version
    /// is the maximum any path requires.
    #[must_use]
    pub fn max(self, other: Version) -> Version {
        if other > self { other } else { self }
    }
}

/// Two versions are equal when they denote the same version.
///
/// The text is kept only so a lock file round-trips: `v1.2.3` and `1.2.3` are
/// the same requirement, and comparing the spelling would make a manifest that
/// wrote one and a lock that wrote the other look like a change.
impl PartialEq for Version {
    fn eq(&self, other: &Version) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl Eq for Version {}

impl std::hash::Hash for Version {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Version::None => 0u8.hash(state),
            Version::Semver { parsed, .. } => {
                1u8.hash(state);
                parsed.hash(state);
            }
        }
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Version) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Version {
    fn cmp(&self, other: &Version) -> Ordering {
        match (self, other) {
            (Version::None, Version::None) => Ordering::Equal,
            (Version::None, Version::Semver { .. }) => Ordering::Less,
            (Version::Semver { .. }, Version::None) => Ordering::Greater,
            (Version::Semver { parsed: a, .. }, Version::Semver { parsed: b, .. }) => a.cmp(b),
        }
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).expect("a version")
    }

    /// [R-MODULE-002] lexicographic ordering would put v0.10.0 below v0.9.0,
    /// which is how a resolution silently picks an older module.
    #[test]
    fn versions_compare_numerically_rather_than_as_text() {
        assert!(v("0.10.0") > v("0.9.0"));
        assert!(v("1.0.0") > v("0.99.99"));
        assert!(v("1.2.3") > v("1.2.2"));
    }

    /// The lock files and manifests both exist in the wild, so both spellings
    /// have to parse and compare.
    #[test]
    fn a_leading_v_is_accepted_and_preserved() {
        assert_eq!(v("v1.2.3"), v("1.2.3"));
        assert_eq!(v("v1.2.3").as_str(), "v1.2.3");
        assert_eq!(v("1.2.3").as_str(), "1.2.3");
    }

    /// [R-MODULE-002] `none` is a value, not an error: it is how a module with
    /// no requirement drops out of a build list.
    #[test]
    fn none_orders_below_every_version() {
        assert!(v("none") < v("0.0.1"));
        assert!(Version::None.is_none());
        assert_eq!(v("").as_str(), "none");
    }

    /// [R-MODULE-003] a typo must fail resolution rather than be skipped,
    /// which would silently drop a dependency.
    #[test]
    fn a_version_that_is_not_semver_fails_and_says_so() {
        let err = Version::parse("1.2.banana").expect_err("should fail");
        assert!(err.contains("1.2.banana"), "{err}");
        assert!(Version::parse("latest").is_err());
    }

    #[test]
    fn max_selects_the_greater() {
        assert_eq!(v("1.0.0").max(v("2.0.0")), v("2.0.0"));
        assert_eq!(v("2.0.0").max(v("1.0.0")), v("2.0.0"));
        assert_eq!(Version::None.max(v("1.0.0")), v("1.0.0"));
    }
}
