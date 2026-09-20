//! The on-disk module cache.
//!
//! `<cache root>/<name>/<key>/`, where the key is the resolved version for a
//! registry module and the commit SHA for a GitHub one. Two configurations on
//! one machine at different versions therefore do not fight; see
//! [R-MODULE-040].
//!
//! The cache is meowctl's own storage rather than the user's configuration, so
//! a dry run fills it for real: a cold machine could otherwise produce no plan
//! at all; see [R-MODULE-045].

use std::collections::BTreeMap;
use std::path::PathBuf;

use meowctl_common::Integrity;
use meowctl_fs::FileSystem;
use meowctl_net::Http;
use serde::{Deserialize, Serialize};

use crate::{ModuleError, ModuleResult, archive, github, registry};

/// The file `v0.1.0` writes beside an extracted module, holding the tarball's
/// SRI.
///
/// Read and written in exactly the format `writeSRISidecar` uses, because the
/// two binaries share one cache directory; see [R-MODULE-044].
const TARBALL_RECORD: &str = ".sri";

/// The file holding one hash per extracted file.
///
/// `v0.1.0` does not write it and ignores it, so a cache it populated has no
/// per-file record and is re-fetched once; see [R-MODULE-042].
const FILE_RECORD: &str = ".files";

/// Where a module comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Source {
    /// A module published in the registry.
    Registry {
        /// The module's name in the index.
        module: String,
        /// The resolved version.
        version: String,
        /// The tarball URL, with the index's template already applied.
        url: String,
        /// The hash the index records for this version, where it records one.
        expected: Option<Integrity>,
    },
    /// A repository at a commit.
    GitHub {
        /// The name this module is known by locally.
        module: String,
        /// Repository owner.
        owner: String,
        /// Repository name.
        repo: String,
        /// The commit, already resolved from the ref; see [R-MODULE-011].
        commit: String,
        /// The archive URL.
        url: String,
    },
}

impl Source {
    /// A registry module at a version, with the index entry's template and
    /// hash applied.
    #[must_use]
    pub fn from_index(module: &str, version: &str, entry: &registry::IndexEntry) -> Source {
        Source::Registry {
            module: module.to_owned(),
            version: version.to_owned(),
            url: registry::source_url(&entry.source, module, version),
            expected: entry.integrity_for(version),
        }
    }

    /// A GitHub repository at an already-resolved commit.
    #[must_use]
    pub fn from_commit(
        module: &str,
        endpoints: &github::GitHubEndpoints,
        owner: &str,
        repo: &str,
        commit: &str,
    ) -> Source {
        Source::GitHub {
            module: module.to_owned(),
            owner: owner.to_owned(),
            repo: repo.to_owned(),
            commit: commit.to_owned(),
            url: endpoints.tarball_url(owner, repo, commit),
        }
    }

    /// The name this module is cached under.
    #[must_use]
    pub fn module(&self) -> &str {
        match self {
            Source::Registry { module, .. } | Source::GitHub { module, .. } => module,
        }
    }

    /// The directory name inside the module's cache: a version, or a commit.
    #[must_use]
    pub fn key(&self) -> &str {
        match self {
            Source::Registry { version, .. } => version,
            Source::GitHub { commit, .. } => commit,
        }
    }

    /// Where the archive is fetched from.
    #[must_use]
    pub fn url(&self) -> &str {
        match self {
            Source::Registry { url, .. } | Source::GitHub { url, .. } => url,
        }
    }

    /// The hash the index promised, when it promised one.
    #[must_use]
    pub const fn expected(&self) -> Option<&Integrity> {
        match self {
            Source::Registry { expected, .. } => expected.as_ref(),
            // GitHub serves a repository, not a published artefact, and there
            // is nothing to compare against until the first fetch. What makes
            // it reproducible is the commit, not a hash; see [R-MODULE-011].
            Source::GitHub { .. } => None,
        }
    }
}

/// What was extracted, and what it hashed to.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct CacheRecord {
    /// One hash per file, keyed by its path inside the module.
    #[serde(default)]
    pub files: BTreeMap<String, String>,
}

/// The module cache.
#[derive(Debug, Clone)]
pub struct Cache {
    root: PathBuf,
}

impl Cache {
    /// A cache rooted at a directory, typically the one
    /// [`meowctl_common::paths::cache_dir`] returns.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Cache { root: root.into() }
    }

    /// Where a module at a key is extracted.
    #[must_use]
    pub fn dir(&self, module: &str, key: &str) -> PathBuf {
        self.root.join(module).join(key)
    }

    /// The tarball hash recorded beside an extracted module.
    ///
    /// # Errors
    ///
    /// Never: an unreadable or malformed record is an absent one, which
    /// [R-MODULE-042] re-fetches.
    #[must_use]
    pub fn tarball_hash(&self, fs: &dyn FileSystem, module: &str, key: &str) -> Option<Integrity> {
        let raw = fs.read(&self.dir(module, key).join(TARBALL_RECORD)).ok()?;
        String::from_utf8(raw).ok()?.trim().parse().ok()
    }

    /// Whether the cache holds this module and every file still hashes to what
    /// was recorded.
    ///
    /// A cache with no record is not verified, per [R-MODULE-042]: it was
    /// extracted by something that did not write one, and what it holds is
    /// unknown.
    #[must_use]
    pub fn is_verified(&self, fs: &dyn FileSystem, module: &str, key: &str) -> bool {
        let Some(record) = self.record(fs, module, key) else {
            return false;
        };
        if record.files.is_empty() {
            return false;
        }
        let dir = self.dir(module, key);
        record.files.iter().all(|(path, expected)| {
            let Ok(expected) = expected.parse::<Integrity>() else {
                return false;
            };
            fs.read(&dir.join(path.replace('/', std::path::MAIN_SEPARATOR_STR)))
                .is_ok_and(|bytes| expected.matches(&bytes))
        })
    }

    /// Makes sure the module is in the cache and verified, fetching it if not.
    ///
    /// Returns the directory it is in. A module already verified is returned
    /// without a request, which is what makes a fully cached lock resolve
    /// offline; see [R-MODULE-041] and [R-MODULE-043].
    ///
    /// # Errors
    ///
    /// [`ModuleError::Fetch`] when the archive cannot be fetched,
    /// [`ModuleError::IntegrityMismatch`] when it is not what the index
    /// promised, and [`ModuleError::Archive`] when it cannot be extracted.
    pub fn ensure(
        &self,
        fs: &dyn FileSystem,
        http: &dyn Http,
        source: &Source,
    ) -> ModuleResult<PathBuf> {
        let module = source.module();
        let key = source.key();
        let dir = self.dir(module, key);
        if self.is_verified(fs, module, key) {
            return Ok(dir);
        }

        let tarball = http.get(source.url()).map_err(|e| ModuleError::Fetch {
            module: module.to_owned(),
            what: "the archive".to_owned(),
            source: e,
        })?;

        // Before extracting, never after: an archive checked afterwards has
        // already written its files somewhere; see [R-MODULE-030].
        if let Some(expected) = source.expected() {
            let actual = Integrity::compute(&tarball);
            if &actual != expected {
                return Err(ModuleError::IntegrityMismatch {
                    module: module.to_owned(),
                    what: "the archive".to_owned(),
                    expected: expected.to_string(),
                    actual: actual.to_string(),
                });
            }
        }

        let files = archive::strip_single_root(archive::read(module, &tarball)?);
        let record = CacheRecord {
            files: files
                .iter()
                .map(|file| {
                    (
                        file.path.clone(),
                        Integrity::compute(&file.contents).to_string(),
                    )
                })
                .collect(),
        };

        // Into a sibling, then renamed. An extraction interrupted partway
        // leaves a directory whose name says what it is, and never one a later
        // run mistakes for a complete module; see [R-MODULE-063].
        let staging = self.root.join(module).join(format!(".incoming-{key}"));
        fs.remove_dir_all(&staging)?;
        archive::write_into(fs, &staging, &files)?;
        fs.write(
            &staging.join(TARBALL_RECORD),
            Integrity::compute(&tarball).to_string().as_bytes(),
        )?;
        fs.write(&staging.join(FILE_RECORD), &encode(module, &record)?)?;

        fs.remove_dir_all(&dir)?;
        fs.create_dir_all(&self.root.join(module))?;
        fs.rename(&staging, &dir)?;
        Ok(dir)
    }

    /// Reads one file out of a cached module, checking it first.
    ///
    /// The check is against the record written at extraction, so a file
    /// changed since is refused rather than evaluated; see [R-STAR-022].
    ///
    /// # Errors
    ///
    /// [`ModuleError::NoSuchFile`] when the module does not hold it, and
    /// [`ModuleError::IntegrityMismatch`] when what is there is not what was
    /// extracted.
    pub fn read_verified(
        &self,
        fs: &dyn FileSystem,
        module: &str,
        key: &str,
        path: &str,
    ) -> ModuleResult<Vec<u8>> {
        let dir = self.dir(module, key);
        let bytes = fs
            .read(&dir.join(path.replace('/', std::path::MAIN_SEPARATOR_STR)))
            .map_err(|_| ModuleError::NoSuchFile {
                module: module.to_owned(),
                path: path.to_owned(),
            })?;
        let Some(expected) = self
            .record(fs, module, key)
            .and_then(|record| record.files.get(path).cloned())
            .and_then(|hash| hash.parse::<Integrity>().ok())
        else {
            return Err(ModuleError::NoSuchFile {
                module: module.to_owned(),
                path: path.to_owned(),
            });
        };
        let actual = Integrity::compute(&bytes);
        if actual != expected {
            return Err(ModuleError::IntegrityMismatch {
                module: module.to_owned(),
                what: path.to_owned(),
                expected: expected.to_string(),
                actual: actual.to_string(),
            });
        }
        Ok(bytes)
    }

    /// The per-file record, when there is a readable one.
    fn record(&self, fs: &dyn FileSystem, module: &str, key: &str) -> Option<CacheRecord> {
        let raw = fs.read(&self.dir(module, key).join(FILE_RECORD)).ok()?;
        toml::from_str(std::str::from_utf8(&raw).ok()?).ok()
    }
}

/// Serialises a record.
///
/// A map of strings to strings always serialises; a failure would mean the
/// `toml` crate changed under us, and it is reported rather than panicked on
/// because nothing here is worth stopping a run over that has not happened.
fn encode(module: &str, record: &CacheRecord) -> ModuleResult<Vec<u8>> {
    toml::to_string(record)
        .map(String::into_bytes)
        .map_err(|e| ModuleError::Archive {
            module: module.to_owned(),
            reason: format!("the cache record could not be written: {e}"),
        })
}
