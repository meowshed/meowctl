//! What a release is, and whether these bytes are the one for this machine.
//!
//! No network and no filesystem. Everything here takes bytes somebody else
//! fetched and returns a decision, which is what makes `self-update`
//! testable without publishing a release to test against; see [R-CLI-070].
//!
//! The one thing this crate is for is the check `v0.1.0` never made.
//! `runSelfUpdate` renames a download over the running binary and its own
//! source says the verification is deferred, so anything that can answer for
//! the URL replaces the user's `meowctl`.

use std::str::FromStr;

use meowctl_common::Integrity;

/// Why a release cannot be used.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum ReleaseError {
    /// The release document did not parse.
    #[error("the release could not be read: {reason}")]
    Unreadable {
        /// What the parser said.
        reason: String,
    },

    /// The release publishes nothing for this machine.
    #[error("{tag} publishes nothing for {platform}; see {url}")]
    NoAsset {
        /// The release's tag.
        tag: String,
        /// The platform that was looked for.
        platform: String,
        /// Where a human can look.
        url: String,
    },

    /// The release publishes no checksums, so nothing can be verified.
    #[error("{tag} publishes no checksums.sri, so nothing about it can be verified")]
    NoChecksums {
        /// The release's tag.
        tag: String,
    },

    /// The checksum file has no line for this asset.
    #[error("checksums.sri has no line for {asset}")]
    Unlisted {
        /// The asset that is missing.
        asset: String,
    },

    /// A checksum line is not one.
    #[error("checksums.sri line {line} is not a hash and a name: {text}")]
    Unparsable {
        /// Which line, counting from one.
        line: usize,
        /// The line itself.
        text: String,
    },

    /// What was downloaded is not what the release published.
    #[error("{asset} does not match what the release published: expected {expected}, got {actual}")]
    Mismatch {
        /// The asset.
        asset: String,
        /// The hash the release published.
        expected: String,
        /// The hash of what arrived.
        actual: String,
    },
}

/// A release, as the forge describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Release {
    /// The tag, as published: `v0.2.0`.
    pub tag: String,
    /// Where a human reads about it.
    pub url: String,
    /// What it publishes, by name.
    pub assets: Vec<Asset>,
}

/// One published file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    /// Its name, which is how a platform's binary is found.
    pub name: String,
    /// Where to fetch it.
    pub url: String,
}

/// The name of the file listing every asset's hash; see [R-CLI-071].
pub const CHECKSUMS: &str = "checksums.sri";

impl Release {
    /// Reads what the GitHub releases API answers with.
    ///
    /// Only the three fields that matter are read, so a field the API adds
    /// is a field this ignores.
    ///
    /// # Errors
    ///
    /// [`ReleaseError::Unreadable`] when the document is not that shape.
    pub fn parse(document: &[u8]) -> Result<Release, ReleaseError> {
        let raw: RawRelease =
            serde_json::from_slice(document).map_err(|e| ReleaseError::Unreadable {
                reason: e.to_string(),
            })?;
        Ok(Release {
            tag: raw.tag_name,
            url: raw.html_url,
            assets: raw
                .assets
                .into_iter()
                .map(|a| Asset {
                    name: a.name,
                    url: a.browser_download_url,
                })
                .collect(),
        })
    }

    /// Whether this release is the version already running.
    ///
    /// A tag is `v0.2.0` and a version is `0.2.0`, which is one `v` apart and
    /// the reason this is a function rather than an equality at the call
    /// site; see [R-CLI-072].
    #[must_use]
    pub fn is(&self, version: &str) -> bool {
        self.tag.strip_prefix('v').unwrap_or(&self.tag) == version
    }

    /// The asset this machine needs.
    ///
    /// # Errors
    ///
    /// [`ReleaseError::NoAsset`] when the release publishes none, naming the
    /// platform and where to look; see [R-CLI-073].
    pub fn asset_for(&self, platform: &str) -> Result<&Asset, ReleaseError> {
        let wanted = format!("meowctl-{platform}");
        self.assets
            .iter()
            .find(|a| a.name == wanted)
            .ok_or_else(|| ReleaseError::NoAsset {
                tag: self.tag.clone(),
                platform: platform.to_owned(),
                url: self.url.clone(),
            })
    }

    /// Where the checksums are.
    ///
    /// # Errors
    ///
    /// [`ReleaseError::NoChecksums`] when the release publishes none. A
    /// release nothing can be verified against is refused rather than
    /// treated as one that needs no verification; see [R-CLI-071].
    pub fn checksums(&self) -> Result<&Asset, ReleaseError> {
        self.assets
            .iter()
            .find(|a| a.name == CHECKSUMS)
            .ok_or_else(|| ReleaseError::NoChecksums {
                tag: self.tag.clone(),
            })
    }
}

/// Checks bytes against what the release published for them.
///
/// The hash is computed here rather than compared as a string somebody else
/// computed, so the only way to pass is to be the bytes.
///
/// # Errors
///
/// [`ReleaseError::Unparsable`] when a line is not a hash and a name,
/// [`ReleaseError::Unlisted`] when the asset has no line, and
/// [`ReleaseError::Mismatch`] when the bytes are not what it names; see
/// [R-CLI-070].
pub fn verify(checksums: &str, asset: &str, bytes: &[u8]) -> Result<(), ReleaseError> {
    let expected = published_hash(checksums, asset)?;
    let actual = Integrity::compute(bytes);
    if expected == actual {
        return Ok(());
    }
    Err(ReleaseError::Mismatch {
        asset: asset.to_owned(),
        expected: expected.as_str().to_owned(),
        actual: actual.as_str().to_owned(),
    })
}

/// The hash a checksum file publishes for one asset.
fn published_hash(checksums: &str, asset: &str) -> Result<Integrity, ReleaseError> {
    for (index, text) in checksums.lines().enumerate() {
        let text = text.trim();
        if text.is_empty() || text.starts_with('#') {
            continue;
        }
        // `<sri>  <name>`, split on the first run of whitespace, because an
        // asset name may not contain one but the separator's width is not
        // ours to fix.
        let (hash, name) =
            text.split_once(char::is_whitespace)
                .ok_or_else(|| ReleaseError::Unparsable {
                    line: index + 1,
                    text: text.to_owned(),
                })?;
        let name = name.trim();
        if name != asset {
            continue;
        }
        return Integrity::from_str(hash).map_err(|_| ReleaseError::Unparsable {
            line: index + 1,
            text: text.to_owned(),
        });
    }
    Err(ReleaseError::Unlisted {
        asset: asset.to_owned(),
    })
}

/// The shape the GitHub releases API answers with.
#[derive(serde::Deserialize)]
struct RawRelease {
    tag_name: String,
    #[serde(default)]
    html_url: String,
    #[serde(default)]
    assets: Vec<RawAsset>,
}

#[derive(serde::Deserialize)]
struct RawAsset {
    name: String,
    browser_download_url: String,
}
