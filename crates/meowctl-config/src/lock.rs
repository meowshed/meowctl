//! `deps.lock` and the package locks.

use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use meowctl_common::Integrity;
use meowctl_fs::FileSystem;

use crate::emit::{self, Value};
use crate::{ConfigError, ConfigResult};

/// What wrote the file and when.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockMeta {
    /// The meowctl version that last wrote it.
    #[serde(rename = "generated-by", default)]
    pub generated_by: String,
    /// An RFC 3339 timestamp of that write.
    #[serde(rename = "updated-at", default)]
    pub updated_at: String,
}

/// One resolved module.
///
/// The key names are `internal/lock/lock.go`'s, and their order is the one a
/// file written by `v0.1.0` has; see [R-CONFIG-020].
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModuleEntry {
    /// The selected semver version, for a registry module.
    #[serde(default)]
    pub version: String,
    /// The canonical URL of the tarball.
    #[serde(default)]
    pub source: String,
    /// The tarball's integrity hash.
    #[serde(default)]
    pub integrity: String,
    /// Each extracted file, relative to the module root, to its own hash.
    ///
    /// Per file rather than per tarball, so a cache mutated after extraction
    /// is detectable; see [R-CONFIG-021].
    #[serde(default)]
    pub files: BTreeMap<String, String>,
    /// The commit a GitHub reference resolved to.
    #[serde(rename = "commit-sha", default)]
    pub commit_sha: String,
    /// Whether a `replace` points this module somewhere local.
    #[serde(default)]
    pub replaced: bool,
    /// Where, when it is replaced.
    #[serde(default)]
    pub path: String,
}

impl ModuleEntry {
    /// What identifies this resolution, for deciding whether it changed.
    ///
    /// The version, then the commit, then the integrity hash.
    /// `moduleFingerprint` is the same chain, and it is a chain rather than
    /// one field so a GitHub module re-synced to a new commit invalidates
    /// even though no version changed; see [R-CONFIG-033].
    #[must_use]
    pub fn fingerprint(&self) -> &str {
        if !self.version.is_empty() {
            return &self.version;
        }
        if !self.commit_sha.is_empty() {
            return &self.commit_sha;
        }
        &self.integrity
    }
}

/// One GitHub module's pin.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitHubEntry {
    /// The commit it resolved to.
    #[serde(default)]
    pub commit: String,
    /// Its integrity hash.
    #[serde(default)]
    pub integrity: String,
}

/// One package's installed state.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct PackageEntry {
    /// The constraint the component asked for.
    #[serde(default)]
    pub requested: String,
    /// What the package manager reported after installing.
    #[serde(default)]
    pub installed: String,
    /// An optional remark.
    #[serde(default)]
    pub note: String,
}

/// A lock file.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct LockFile {
    /// Provenance.
    #[serde(default)]
    pub meta: LockMeta,
    /// Registry and GitHub modules, by name.
    #[serde(default)]
    pub modules: BTreeMap<String, ModuleEntry>,
    /// GitHub pins, by `github.com/owner/repo`.
    #[serde(rename = "github-modules", default)]
    pub github: BTreeMap<String, GitHubEntry>,
    /// Packages, by manager and then by package.
    #[serde(default)]
    pub packages: BTreeMap<String, BTreeMap<String, PackageEntry>>,
}

impl LockFile {
    /// Reads a lock file.
    ///
    /// A file that is not there is an empty lock rather than an error: a first
    /// run has none, and treating that as a failure would make `init` the only
    /// command that works; see [R-CONFIG-003].
    ///
    /// # Errors
    ///
    /// [`ConfigError::Malformed`] when the file is not valid TOML.
    pub fn read(fs: &dyn FileSystem, path: &Path) -> ConfigResult<LockFile> {
        let bytes = match fs.read(path) {
            Ok(b) => b,
            Err(meowctl_fs::FsError::NotFound { .. }) => return Ok(LockFile::default()),
            Err(e) => return Err(e.into()),
        };
        let text = String::from_utf8_lossy(&bytes);
        let lock: LockFile = toml::from_str(&text).map_err(|e| ConfigError::Malformed {
            path: path.to_path_buf(),
            reason: e.to_string(),
        })?;
        lock.check_hashes(path)?;
        Ok(lock)
    }

    /// Refuses a hash that is not a W3C Subresource Integrity one.
    ///
    /// The field is a `String` rather than an [`Integrity`] because a
    /// replaced module has no hash and an empty string is not one. Checking
    /// here gives the same guarantee at the same moment: a malformed hash
    /// fails where the file is read, rather than comparing unequal forever
    /// and looking like tampering; see [R-CONFIG-061] and [R-COMMON-004].
    fn check_hashes(&self, path: &Path) -> ConfigResult<()> {
        let check = |what: &str, value: &str| -> ConfigResult<()> {
            if value.is_empty() || value.parse::<Integrity>().is_ok() {
                return Ok(());
            }
            Err(ConfigError::Malformed {
                path: path.to_path_buf(),
                reason: format!("{what} is not a sha384- hash: {value}"),
            })
        };

        for (name, entry) in &self.modules {
            check(&format!("the integrity of {name}"), &entry.integrity)?;
            for (file, hash) in &entry.files {
                check(&format!("the integrity of {name}'s {file}"), hash)?;
            }
        }
        for (name, entry) in &self.github {
            check(&format!("the integrity of {name}"), &entry.integrity)?;
        }
        Ok(())
    }

    /// Writes a lock file in the layout `v0.1.0` emits.
    ///
    /// # Errors
    ///
    /// Whatever the filesystem fails with.
    pub fn write(&self, fs: &dyn FileSystem, path: &Path) -> ConfigResult<()> {
        fs.write(path, emit::to_string(&self.to_value()).as_bytes())?;
        Ok(())
    }

    /// The document, in the order `v0.1.0` writes it.
    fn to_value(&self) -> Value {
        Value::Table(vec![
            (
                "meta".to_owned(),
                Value::Table(vec![
                    (
                        "generated-by".to_owned(),
                        Value::String(self.meta.generated_by.clone()),
                    ),
                    (
                        "updated-at".to_owned(),
                        Value::String(self.meta.updated_at.clone()),
                    ),
                ]),
            ),
            ("modules".to_owned(), modules_value(&self.modules)),
            ("github-modules".to_owned(), github_value(&self.github)),
            ("packages".to_owned(), packages_value(&self.packages)),
        ])
    }

    /// Merges a local lock over this one.
    ///
    /// An entry in both wins from the local file, because the local one is how
    /// a machine differs from the committed configuration; see [R-CONFIG-024].
    #[must_use]
    pub fn overlaid_with(&self, local: &LockFile) -> LockFile {
        let mut merged = self.clone();
        merged.modules.extend(local.modules.clone());
        merged.github.extend(local.github.clone());
        for (manager, packages) in &local.packages {
            merged
                .packages
                .entry(manager.clone())
                .or_default()
                .extend(packages.clone());
        }
        merged
    }
}

fn modules_value(modules: &BTreeMap<String, ModuleEntry>) -> Value {
    Value::Table(
        modules
            .iter()
            .map(|(name, entry)| {
                (
                    name.clone(),
                    Value::table(vec![
                        ("version", Some(Value::String(entry.version.clone()))),
                        ("source", Some(Value::String(entry.source.clone()))),
                        ("integrity", Some(Value::String(entry.integrity.clone()))),
                        (
                            "files",
                            Some(Value::Table(
                                entry
                                    .files
                                    .iter()
                                    .map(|(p, h)| (p.clone(), Value::String(h.clone())))
                                    .collect(),
                            )),
                        ),
                        (
                            "commit-sha",
                            emit::Value::optional_string(&entry.commit_sha),
                        ),
                        ("replaced", entry.replaced.then_some(Value::Boolean(true))),
                        ("path", emit::Value::optional_string(&entry.path)),
                    ]),
                )
            })
            .collect(),
    )
}

fn github_value(entries: &BTreeMap<String, GitHubEntry>) -> Value {
    Value::Table(
        entries
            .iter()
            .map(|(name, entry)| {
                (
                    name.clone(),
                    Value::Table(vec![
                        ("commit".to_owned(), Value::String(entry.commit.clone())),
                        (
                            "integrity".to_owned(),
                            Value::String(entry.integrity.clone()),
                        ),
                    ]),
                )
            })
            .collect(),
    )
}

fn packages_value(packages: &BTreeMap<String, BTreeMap<String, PackageEntry>>) -> Value {
    Value::Table(
        packages
            .iter()
            .map(|(manager, entries)| {
                (
                    manager.clone(),
                    Value::Table(
                        entries
                            .iter()
                            .map(|(name, entry)| {
                                (
                                    name.clone(),
                                    Value::table(vec![
                                        ("requested", Some(Value::String(entry.requested.clone()))),
                                        ("installed", Some(Value::String(entry.installed.clone()))),
                                        ("note", emit::Value::optional_string(&entry.note)),
                                    ]),
                                )
                            })
                            .collect(),
                    ),
                )
            })
            .collect(),
    )
}
