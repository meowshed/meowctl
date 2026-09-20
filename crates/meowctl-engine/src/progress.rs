//! What has been done, and what has to be done again.
//!
//! Two questions about the same file. The sentinel says which components
//! finished which phases, so a run does not redo them; staleness says which of
//! those records to throw away because the module the component came from has
//! changed under it.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use meowctl_common::ComponentId;
use meowctl_config::Sentinel;
use meowctl_fs::FileSystem;

use crate::{EngineError, EngineResult};

/// The sentinel, and where it is written.
#[derive(Debug, Clone)]
pub struct Progress {
    sentinel: Sentinel,
    path: PathBuf,
    /// What to stamp a record with.
    ///
    /// A value rather than a reading of the clock, because a component that
    /// asks the clock cannot be tested; see [R-ENGINE-041].
    now: Option<toml::value::Datetime>,
}

impl Progress {
    /// Progress over this sentinel, written to this path.
    #[must_use]
    pub fn new(sentinel: Sentinel, path: impl Into<PathBuf>) -> Self {
        Progress {
            sentinel,
            path: path.into(),
            now: None,
        }
    }

    /// Stamps records with this time.
    #[must_use]
    pub fn at(mut self, now: toml::value::Datetime) -> Self {
        self.now = Some(now);
        self
    }

    /// What has been done.
    #[must_use]
    pub const fn sentinel(&self) -> &Sentinel {
        &self.sentinel
    }

    /// Records a component as having finished a phase, and writes the file.
    ///
    /// Written immediately rather than at the end of the phase, because the
    /// point of recording it is that an interrupted run resumes where it
    /// stopped, and a record still in memory when the process dies records
    /// nothing; see [R-ENGINE-041] and [R-ENGINE-062].
    ///
    /// # Errors
    ///
    /// [`EngineError::Configuration`] when the file cannot be written.
    pub fn record(
        &mut self,
        fs: &dyn FileSystem,
        phase: &str,
        component: &str,
    ) -> EngineResult<()> {
        self.sentinel.record(phase, component, self.now);
        self.flush(fs)
    }

    /// Throws away everything recorded for these components.
    ///
    /// # Errors
    ///
    /// [`EngineError::Configuration`] when the file cannot be written.
    pub fn forget(&mut self, fs: &dyn FileSystem, components: &[String]) -> EngineResult<()> {
        if components.is_empty() {
            return Ok(());
        }
        for component in components {
            self.sentinel.forget(component);
        }
        self.flush(fs)
    }

    /// Writes the file.
    fn flush(&self, fs: &dyn FileSystem) -> EngineResult<()> {
        self.sentinel
            .write(fs, &self.path)
            .map_err(|e| EngineError::Configuration {
                path: self.path.display().to_string(),
                reason: e.to_string(),
            })
    }
}

/// Which components must be done again because their module changed.
///
/// A component belongs to a module, and a module has a fingerprint: its
/// version, its commit, or its integrity hash. When the fingerprint recorded
/// in `installed.lock` differs from what the lock resolves now, every
/// component of that module is stale -- including the ones `installed.lock`
/// does not name, which are the transitive ones an aggregate brought in.
///
/// `computeStaleComponents` is the same computation, and
/// `fix: correct module updates` is why it exists: without it a bumped module
/// reported its components already installed and left the symlinks pointing
/// at the old cached version; see [R-ENGINE-043] and [R-ENGINE-044].
#[must_use]
pub fn stale_components(
    components: &[ComponentId],
    recorded: &BTreeMap<String, String>,
    current: &BTreeMap<String, String>,
) -> Vec<String> {
    // A module whose fingerprint moved. A component `installed.lock` records
    // but that the configuration no longer resolves is not a change: it is
    // gone, and uninstalling is a different command.
    let mut changed: BTreeSet<&str> = BTreeSet::new();
    for component in components {
        let name = component.logical_name();
        let (Some(was), Some(now)) = (recorded.get(name), current.get(name)) else {
            continue;
        };
        if was != now {
            changed.insert(component.module_key());
        }
    }
    if changed.is_empty() {
        return Vec::new();
    }

    components
        .iter()
        .filter(|component| changed.contains(component.module_key()))
        .map(|component| component.logical_name().to_owned())
        .collect()
}

/// Which module each component resolves from, as a fingerprint.
///
/// Components that belong to no module are absent, because a bare component
/// has nothing that can go stale.
#[must_use]
pub fn fingerprints(
    components: &[ComponentId],
    lock: &meowctl_config::LockFile,
) -> BTreeMap<String, String> {
    components
        .iter()
        .filter(|component| component.is_module_qualified())
        .filter_map(|component| {
            let entry = lock.modules.get(component.module_key())?;
            Some((
                component.logical_name().to_owned(),
                entry.fingerprint().to_owned(),
            ))
        })
        .collect()
}

/// Whether a journal left by an earlier run is waiting to be replayed.
///
/// Reported before anything else happens, because a non-empty journal means
/// the last run stopped partway and the machine is in a state nobody chose;
/// see [R-ENGINE-042] and [R-OPS-025].
///
/// # Errors
///
/// Never: a journal that cannot be read is reported as absent, and the run
/// that finds it broken will say so.
#[must_use]
pub fn interrupted_run(path: &Path) -> Option<usize> {
    match meowctl_ops::peek(path) {
        Ok(0) | Err(_) => None,
        Ok(records) => Some(records),
    }
}
