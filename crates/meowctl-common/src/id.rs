//! Identifiers: components, modules, and integrity hashes.

use std::fmt;
use std::str::FromStr;

use base64::Engine as _;
use base64::engine::general_purpose::STANDARD;
use sha2::{Digest as _, Sha384};

use serde::{Deserialize, Serialize};

use crate::Error;

/// A component, in one of the three forms a configuration may name it.
///
/// The three are a bare name (`neovim`), a registry-qualified path
/// (`@stdlib//components/zsh`) or module root (`@dotmeow`), and a
/// GitHub-qualified path (`github.com/owner/repo//components/zsh`). The
/// original text is kept so `Display` reproduces what was written; see
/// [R-COMMON-001].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ComponentId(String);

impl ComponentId {
    /// The identifier as written.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The key under which this component's module appears in a lock file.
    ///
    /// `@dotmeow//components/x` and `@dotmeow` both key on `dotmeow`;
    /// `github.com/o/r//x` keys on `github.com/o/r`. A bare name keys on
    /// itself, which matches no module entry, and that is how a local
    /// component is distinguished from one that came from a module.
    /// `moduleKeyFromComponentURL` in `internal/cli/apply.go` is the same
    /// computation; see [R-COMMON-002].
    #[must_use]
    pub fn module_key(&self) -> &str {
        let s = self.0.strip_prefix('@').unwrap_or(&self.0);
        match s.find("//") {
            Some(i) => &s[..i],
            None => s,
        }
    }

    /// Whether this component came from a module rather than from the
    /// configuration directory.
    #[must_use]
    pub fn is_module_qualified(&self) -> bool {
        self.0.starts_with('@') || self.0.contains("//")
    }
}

impl FromStr for ComponentId {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || Error::InvalidComponentId {
            value: s.to_owned(),
        };

        if s.is_empty() || s.trim() != s {
            return Err(invalid());
        }
        // `@` alone, or `@//path`, names no module.
        if s == "@" || s.starts_with("@//") {
            return Err(invalid());
        }
        // A trailing `//` names no path inside the module.
        if s.ends_with("//") {
            return Err(invalid());
        }
        Ok(ComponentId(s.to_owned()))
    }
}

impl fmt::Display for ComponentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// Where a module comes from.
///
/// A registry module is named by a bare identifier and resolved through the
/// registry index. A GitHub module carries its owner, repository, and the tag
/// or branch to resolve; see [R-COMMON-003].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModuleRef {
    /// A module resolved through the registry index.
    Registry {
        /// The module's name in the index.
        name: String,
    },
    /// A module fetched from GitHub at a tag or branch.
    GitHub {
        /// Repository owner.
        owner: String,
        /// Repository name.
        repo: String,
        /// The tag or branch to resolve to a commit.
        reference: String,
    },
}

impl ModuleRef {
    /// The key this module appears under in a lock file.
    #[must_use]
    pub fn lock_key(&self) -> String {
        match self {
            ModuleRef::Registry { name } => name.clone(),
            ModuleRef::GitHub { owner, repo, .. } => format!("github.com/{owner}/{repo}"),
        }
    }
}

impl FromStr for ModuleRef {
    type Err = Error;

    /// Accepts `name` and `github:owner/repo@ref`. Anything else fails rather
    /// than becoming a registry module with a strange name, which would turn
    /// a typo into a confusing index lookup.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || Error::InvalidModuleRef {
            value: s.to_owned(),
        };

        if let Some(rest) = s.strip_prefix("github:") {
            let (path, reference) = rest.rsplit_once('@').ok_or_else(invalid)?;
            let (owner, repo) = path.split_once('/').ok_or_else(invalid)?;
            if owner.is_empty() || repo.is_empty() || repo.contains('/') || reference.is_empty() {
                return Err(invalid());
            }
            return Ok(ModuleRef::GitHub {
                owner: owner.to_owned(),
                repo: repo.to_owned(),
                reference: reference.to_owned(),
            });
        }

        if s.is_empty()
            || !s
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
        {
            return Err(invalid());
        }
        Ok(ModuleRef::Registry { name: s.to_owned() })
    }
}

impl fmt::Display for ModuleRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ModuleRef::Registry { name } => f.write_str(name),
            ModuleRef::GitHub {
                owner,
                repo,
                reference,
            } => {
                write!(f, "github:{owner}/{repo}@{reference}")
            }
        }
    }
}

/// A W3C Subresource Integrity hash, as the lock files store it.
///
/// Validated on construction so a malformed hash fails where it is read rather
/// than comparing unequal forever and looking like tampering; see
/// [R-COMMON-004].
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Integrity(String);

impl Integrity {
    /// The hash as it appears in a lock file.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The hash of some bytes: SHA-384 in standard base64.
    ///
    /// What `computeSRI` in `internal/starlark/loader/github.go` produces, and
    /// what every published `index.toml` carries. Here rather than at the two
    /// call sites that hash bytes, so there is one encoding of one hash; see
    /// [R-COMMON-005].
    #[must_use]
    pub fn compute(data: &[u8]) -> Integrity {
        let digest = Sha384::digest(data);
        Integrity(format!("sha384-{}", STANDARD.encode(digest)))
    }

    /// Whether some bytes hash to this.
    ///
    /// `v0.1.0` decodes the base64 and compares digests. Comparing the strings
    /// agrees with that whenever both sides came from [`Integrity::compute`],
    /// and makes a hash recorded in another encoding a mismatch rather than a
    /// silent success.
    #[must_use]
    pub fn matches(&self, data: &[u8]) -> bool {
        &Integrity::compute(data) == self
    }
}

impl FromStr for Integrity {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let invalid = || Error::InvalidIntegrity {
            value: s.to_owned(),
        };
        let digest = s.strip_prefix("sha384-").ok_or_else(invalid)?;
        if digest.is_empty()
            || !digest
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        {
            return Err(invalid());
        }
        Ok(Integrity(s.to_owned()))
    }
}

impl fmt::Display for Integrity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [R-COMMON-001] all three forms are in real configurations, and Display
    /// reproduces what was written so a message quotes the user's own text.
    #[test]
    fn the_three_component_forms_round_trip() {
        for raw in [
            "neovim",
            "@dotmeow",
            "@stdlib//components/zsh",
            "github.com/meowshed/meowctl-stdlib//components/zsh",
        ] {
            let id: ComponentId = raw.parse().unwrap();
            assert_eq!(id.to_string(), raw);
        }
    }

    /// [R-COMMON-002] the key is what a lock lookup uses, and getting the
    /// `//` split wrong silently stops a module bump from invalidating.
    #[test]
    fn the_module_key_matches_v0_1_0() {
        let key = |s: &str| s.parse::<ComponentId>().unwrap().module_key().to_owned();
        assert_eq!(key("@dotmeow//components/x"), "dotmeow");
        assert_eq!(key("@dotmeow"), "dotmeow");
        assert_eq!(key("github.com/o/r//components/x"), "github.com/o/r");
        assert_eq!(key("neovim"), "neovim");
    }

    #[test]
    fn a_bare_name_is_not_module_qualified() {
        assert!(
            !"neovim"
                .parse::<ComponentId>()
                .unwrap()
                .is_module_qualified()
        );
        assert!(
            "@dotmeow"
                .parse::<ComponentId>()
                .unwrap()
                .is_module_qualified()
        );
        assert!(
            "github.com/o/r//x"
                .parse::<ComponentId>()
                .unwrap()
                .is_module_qualified()
        );
    }

    /// [R-COMMON-050] these strings come from files a user edits, so a
    /// malformed one returns an error rather than panicking.
    #[test]
    fn malformed_component_identifiers_are_rejected() {
        for raw in ["", "@", "@//path", "@stdlib//", " neovim", "neovim "] {
            assert!(raw.parse::<ComponentId>().is_err(), "accepted {raw:?}");
        }
    }

    /// [R-COMMON-003] the two source kinds resolve differently, so a string
    /// that is neither must not become a registry module with a strange name.
    #[test]
    fn module_references_parse_both_forms() {
        assert_eq!(
            "stdlib".parse::<ModuleRef>().unwrap(),
            ModuleRef::Registry {
                name: "stdlib".to_owned()
            }
        );
        assert_eq!(
            "github:meowshed/meowctl-stdlib@v1.2.3"
                .parse::<ModuleRef>()
                .unwrap(),
            ModuleRef::GitHub {
                owner: "meowshed".to_owned(),
                repo: "meowctl-stdlib".to_owned(),
                reference: "v1.2.3".to_owned(),
            }
        );
    }

    /// A GitHub reference may be a branch, and a branch may contain a slash,
    /// which is why the reference is split from the right.
    #[test]
    fn a_github_reference_may_contain_a_slash() {
        let r = "github:o/r@feature/thing".parse::<ModuleRef>().unwrap();
        assert_eq!(
            r,
            ModuleRef::GitHub {
                owner: "o".to_owned(),
                repo: "r".to_owned(),
                reference: "feature/thing".to_owned(),
            }
        );
    }

    #[test]
    fn malformed_module_references_are_rejected() {
        for raw in [
            "",
            "github:",
            "github:owner@ref",
            "github:o/r",
            "github:o/r@",
            "has space",
            "a/b",
        ] {
            assert!(raw.parse::<ModuleRef>().is_err(), "accepted {raw:?}");
        }
    }

    #[test]
    fn a_module_reference_keys_a_lock_entry() {
        assert_eq!("stdlib".parse::<ModuleRef>().unwrap().lock_key(), "stdlib");
        assert_eq!(
            "github:o/r@v1".parse::<ModuleRef>().unwrap().lock_key(),
            "github.com/o/r"
        );
    }

    /// [R-COMMON-004] an unvalidated hash compares unequal forever and looks
    /// like tampering, so it fails where it is read instead.
    #[test]
    fn integrity_hashes_are_validated_on_construction() {
        assert!("sha384-abc123+/=".parse::<Integrity>().is_ok());
        for raw in ["", "sha384-", "sha256-abc", "abc123", "sha384-abc!"] {
            assert!(raw.parse::<Integrity>().is_err(), "accepted {raw:?}");
        }
    }

    /// [R-COMMON-005] the expected value comes from `sha384sum` rather than
    /// from either implementation, so the test checks the algorithm and not
    /// one program's opinion of it.
    #[test]
    fn the_hash_is_sha384_in_standard_base64() {
        assert_eq!(
            Integrity::compute(b"meowctl\n").as_str(),
            "sha384-sHD4keTTvwX8wx5/hCy2/TVcFU1hORRQylCB/BOwkey8SPs7BxZw4NFSd5q6uEIq"
        );
    }

    /// A computed hash is one a lock file could hold, which is what lets the
    /// same type carry both.
    #[test]
    fn a_computed_hash_is_a_valid_one() {
        let computed = Integrity::compute(b"");
        assert!(computed.as_str().parse::<Integrity>().is_ok());
    }

    #[test]
    fn a_changed_byte_is_a_mismatch() {
        let sri = Integrity::compute(b"one");
        assert!(sri.matches(b"one"));
        assert!(!sri.matches(b"onf"));
    }
}
