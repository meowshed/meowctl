//! The implementation that predicts a run instead of performing it.
//!
//! A dry run is an implementation of [`FileSystem`], not a flag checked in
//! every effectful function. That is the whole point: `v0.1.0` checks
//! `c.caps.DryRun` in a dozen places and missed one, which is the bug
//! `fix(apply): dry-run claimed work that the runner skips` closed.
//!
//! Two things make the prediction worth trusting. A read of a path this run
//! recorded a write for answers with what was written, so a hook that writes a
//! file and reads it back takes the same branch it will take for real; see
//! [R-FS-012]. And a write whose directory does not exist fails here too,
//! because a dry run that succeeds where the real run fails is worse than
//! none; see [R-FS-033].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::{Displaced, Entry, FileSystem, FsError, FsResult};

/// What the run would have done to a path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Intent {
    /// The file's contents would be replaced.
    Wrote {
        /// What would have been written.
        contents: Vec<u8>,
    },
    /// The path would be removed.
    Removed,
    /// A symlink would be created here.
    Linked {
        /// What it would point at.
        target: PathBuf,
    },
    /// A directory would be created.
    CreatedDirectory,
}

/// Records what a run would do and performs none of it.
#[derive(Debug)]
pub struct DryRunFs {
    /// Reads fall through to this, so a hook sees the machine as it is.
    underlying: Box<dyn FileSystem + Send + Sync>,
    intents: Mutex<BTreeMap<PathBuf, Intent>>,
}

impl DryRunFs {
    /// Wraps a filesystem, reading through it and writing nowhere.
    #[must_use]
    pub fn new(underlying: Box<dyn FileSystem + Send + Sync>) -> Self {
        DryRunFs {
            underlying,
            intents: Mutex::new(BTreeMap::new()),
        }
    }

    /// What the run would have done, in path order.
    ///
    /// # Panics
    ///
    /// If the lock is poisoned.
    #[must_use]
    pub fn intents(&self) -> Vec<(PathBuf, Intent)> {
        self.lock()
            .iter()
            .map(|(p, i)| (p.clone(), i.clone()))
            .collect()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<PathBuf, Intent>> {
        self.intents
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn record(&self, path: &Path, intent: Intent) {
        self.lock().insert(path.to_path_buf(), intent);
    }

    /// Whether a directory would exist once this run's intents are applied.
    ///
    /// A write into a directory an earlier step created must succeed, or the
    /// plan stops halfway through a component that would have worked.
    fn directory_would_exist(&self, path: &Path) -> FsResult<bool> {
        if matches!(self.lock().get(path), Some(Intent::CreatedDirectory)) {
            return Ok(true);
        }
        Ok(self.underlying.entry(path)?.is_some_and(|e| e.is_dir()))
    }

    fn require_parent(&self, path: &Path) -> FsResult<()> {
        let Some(parent) = path.parent() else {
            return Ok(());
        };
        if parent.as_os_str().is_empty() || self.directory_would_exist(parent)? {
            return Ok(());
        }
        Err(FsError::NoParent {
            path: path.to_path_buf(),
            parent: parent.to_path_buf(),
        })
    }
}

impl FileSystem for DryRunFs {
    fn read(&self, path: &Path) -> FsResult<Vec<u8>> {
        match self.lock().get(path) {
            Some(Intent::Wrote { contents }) => Ok(contents.clone()),
            Some(Intent::Removed) => Err(FsError::NotFound {
                path: path.to_path_buf(),
            }),
            _ => self.underlying.read(path),
        }
    }

    fn write(&self, path: &Path, contents: &[u8]) -> FsResult<()> {
        self.require_parent(path)?;
        self.record(
            path,
            Intent::Wrote {
                contents: contents.to_vec(),
            },
        );
        Ok(())
    }

    fn append(&self, path: &Path, contents: &[u8]) -> FsResult<()> {
        self.require_parent(path)?;
        let mut existing = match self.read(path) {
            Ok(bytes) => bytes,
            Err(FsError::NotFound { .. }) => Vec::new(),
            Err(e) => return Err(e),
        };
        existing.extend_from_slice(contents);
        self.record(path, Intent::Wrote { contents: existing });
        Ok(())
    }

    fn remove(&self, path: &Path) -> FsResult<()> {
        self.record(path, Intent::Removed);
        Ok(())
    }

    fn copy(&self, from: &Path, to: &Path) -> FsResult<()> {
        let contents = self.read(from)?;
        self.require_parent(to)?;
        self.record(to, Intent::Wrote { contents });
        Ok(())
    }

    fn symlink(&self, target: &Path, link: &Path, backup: Option<&Path>) -> FsResult<Displaced> {
        self.require_parent(link)?;
        let displaced = match self.entry(link)? {
            None => Displaced::Nothing,
            Some(Entry::Symlink { target }) => Displaced::Symlink { target },
            Some(_) => {
                let Some(backup) = backup else {
                    return Err(FsError::WouldClobber {
                        path: link.to_path_buf(),
                    });
                };
                Displaced::BackedUp {
                    backup: backup.to_path_buf(),
                }
            }
        };
        self.record(
            link,
            Intent::Linked {
                target: target.to_path_buf(),
            },
        );
        Ok(displaced)
    }

    fn read_link(&self, path: &Path) -> FsResult<PathBuf> {
        match self.lock().get(path) {
            Some(Intent::Linked { target }) => Ok(target.clone()),
            Some(Intent::Removed) => Err(FsError::NotFound {
                path: path.to_path_buf(),
            }),
            Some(_) => Err(FsError::NotASymlink {
                path: path.to_path_buf(),
            }),
            None => self.underlying.read_link(path),
        }
    }

    fn remove_symlink(&self, path: &Path) -> FsResult<()> {
        match self.entry(path)? {
            Some(Entry::Symlink { .. }) => {
                self.record(path, Intent::Removed);
                Ok(())
            }
            Some(_) => Err(FsError::NotASymlink {
                path: path.to_path_buf(),
            }),
            None => Err(FsError::NotFound {
                path: path.to_path_buf(),
            }),
        }
    }

    fn create_dir_all(&self, path: &Path) -> FsResult<bool> {
        if self.directory_would_exist(path)? {
            return Ok(false);
        }
        let mut dir = Some(path);
        while let Some(d) = dir {
            if self.underlying.entry(d)?.is_some_and(|e| e.is_dir()) {
                break;
            }
            self.record(d, Intent::CreatedDirectory);
            dir = d.parent();
        }
        Ok(true)
    }

    fn read_dir(&self, path: &Path) -> FsResult<Vec<PathBuf>> {
        self.underlying.read_dir(path)
    }

    fn rename(&self, from: &Path, to: &Path) -> FsResult<()> {
        let contents = self.read(from)?;
        self.require_parent(to)?;
        self.record(from, Intent::Removed);
        self.record(to, Intent::Wrote { contents });
        Ok(())
    }

    fn entry(&self, path: &Path) -> FsResult<Option<Entry>> {
        match self.lock().get(path) {
            Some(Intent::Wrote { contents }) => Ok(Some(Entry::File {
                len: contents.len() as u64,
            })),
            Some(Intent::Removed) => Ok(None),
            Some(Intent::Linked { target }) => Ok(Some(Entry::Symlink {
                target: target.clone(),
            })),
            Some(Intent::CreatedDirectory) => Ok(Some(Entry::Directory)),
            None => self.underlying.entry(path),
        }
    }
}
