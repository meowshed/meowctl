//! A filesystem held in memory.
//!
//! It exists so a test of the engine, the operations, or a hook needs no
//! temporary directory to create and clean up, and so two tests can run in
//! parallel without sharing a tree; see [R-FS-013].

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use crate::{Displaced, Entry, FileSystem, FsError, FsResult};

/// What a path holds.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Node {
    File {
        bytes: Vec<u8>,
        /// Tracked so an extraction that sets the bit can be asserted on, and
        /// so a snapshot notices when an undo does not put it back; see
        /// [R-FS-005].
        executable: bool,
    },
    Directory,
    Symlink(PathBuf),
}

/// A filesystem in memory.
///
/// Paths are stored as given. Callers hand it absolute paths, because
/// resolution happens before the filesystem sees anything; see [R-FS-002].
#[derive(Debug, Default)]
pub struct MemFs {
    nodes: Mutex<BTreeMap<PathBuf, Node>>,
}

impl MemFs {
    /// An empty filesystem with a root directory.
    #[must_use]
    pub fn new() -> Self {
        let mut nodes = BTreeMap::new();
        nodes.insert(PathBuf::from("/"), Node::Directory);
        MemFs {
            nodes: Mutex::new(nodes),
        }
    }

    /// Seeds a file, creating its directories, for a test that needs a tree to
    /// start from.
    ///
    /// # Panics
    ///
    /// If the lock is poisoned, which means another test panicked while
    /// holding it.
    pub fn seed(&self, path: impl AsRef<Path>, contents: impl AsRef<[u8]>) {
        let path = path.as_ref().to_path_buf();
        let mut nodes = self.lock();
        let mut dir = path.parent();
        while let Some(d) = dir {
            nodes.entry(d.to_path_buf()).or_insert(Node::Directory);
            dir = d.parent();
        }
        nodes.insert(
            path,
            Node::File {
                bytes: contents.as_ref().to_vec(),
                executable: false,
            },
        );
    }

    /// Every path, sorted, for a test that asserts on the whole tree.
    ///
    /// # Panics
    ///
    /// If the lock is poisoned.
    #[must_use]
    pub fn paths(&self) -> Vec<PathBuf> {
        self.lock().keys().cloned().collect()
    }

    /// A snapshot that can be compared with another, for a test that applies
    /// an operation and then undoes it; see [R-OPS-004].
    ///
    /// # Panics
    ///
    /// If the lock is poisoned.
    #[must_use]
    pub fn snapshot(&self) -> Vec<(PathBuf, String)> {
        self.lock()
            .iter()
            .map(|(path, node)| {
                let described = match node {
                    Node::File { bytes, executable } => format!(
                        "{}:{}",
                        if *executable { "exec" } else { "file" },
                        String::from_utf8_lossy(bytes)
                    ),
                    Node::Directory => "dir".to_owned(),
                    Node::Symlink(target) => format!("link:{}", target.display()),
                };
                (path.clone(), described)
            })
            .collect()
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<PathBuf, Node>> {
        self.nodes
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    fn require_parent(nodes: &BTreeMap<PathBuf, Node>, path: &Path) -> FsResult<()> {
        let Some(parent) = path.parent() else {
            return Ok(());
        };
        if parent.as_os_str().is_empty() || matches!(nodes.get(parent), Some(Node::Directory)) {
            return Ok(());
        }
        Err(FsError::NoParent {
            path: path.to_path_buf(),
            parent: parent.to_path_buf(),
        })
    }
}

/// Whether anything is stored under `path`.
fn has_children(nodes: &BTreeMap<PathBuf, Node>, path: &Path) -> bool {
    nodes
        .keys()
        .any(|held| held != path && held.starts_with(path))
}

impl FileSystem for MemFs {
    fn read(&self, path: &Path) -> FsResult<Vec<u8>> {
        match self.lock().get(path) {
            Some(Node::File { bytes, .. }) => Ok(bytes.clone()),
            Some(_) | None => Err(FsError::NotFound {
                path: path.to_path_buf(),
            }),
        }
    }

    fn write(&self, path: &Path, contents: &[u8]) -> FsResult<()> {
        let mut nodes = self.lock();
        Self::require_parent(&nodes, path)?;
        // A replacement keeps the bit: `RealFs` rewrites the contents and
        // leaves the mode, and a memory filesystem that disagreed would hide
        // that difference from every test.
        let executable = matches!(
            nodes.get(path),
            Some(Node::File {
                executable: true,
                ..
            })
        );
        nodes.insert(
            path.to_path_buf(),
            Node::File {
                bytes: contents.to_vec(),
                executable,
            },
        );
        Ok(())
    }

    fn append(&self, path: &Path, contents: &[u8]) -> FsResult<()> {
        let mut nodes = self.lock();
        Self::require_parent(&nodes, path)?;
        match nodes.get_mut(path) {
            Some(Node::File { bytes, .. }) => bytes.extend_from_slice(contents),
            _ => {
                nodes.insert(
                    path.to_path_buf(),
                    Node::File {
                        bytes: contents.to_vec(),
                        executable: false,
                    },
                );
            }
        }
        Ok(())
    }

    fn remove(&self, path: &Path) -> FsResult<()> {
        let mut nodes = self.lock();
        if matches!(nodes.get(path), Some(Node::Directory)) && has_children(&nodes, path) {
            return Err(FsError::io(
                "removing",
                path,
                std::io::Error::new(
                    std::io::ErrorKind::DirectoryNotEmpty,
                    "the directory is not empty",
                ),
            ));
        }
        nodes.remove(path);
        Ok(())
    }

    fn remove_dir_all(&self, path: &Path) -> FsResult<()> {
        let mut nodes = self.lock();
        nodes.retain(|held, _| held != path && !held.starts_with(path));
        Ok(())
    }

    fn copy(&self, from: &Path, to: &Path) -> FsResult<()> {
        let mut nodes = self.lock();
        let Some(Node::File { bytes, .. }) = nodes.get(from).cloned() else {
            return Err(FsError::NotFound {
                path: from.to_path_buf(),
            });
        };
        Self::require_parent(&nodes, to)?;
        // A copy creates a file, and [R-FS-004] gives a created file `0o600`:
        // `RealFs` sets the mode after `fs::copy`, so the bit does not travel.
        // A rename moves the file and does carry it.
        nodes.insert(
            to.to_path_buf(),
            Node::File {
                bytes,
                executable: false,
            },
        );
        Ok(())
    }

    fn symlink(&self, target: &Path, link: &Path, backup: Option<&Path>) -> FsResult<Displaced> {
        let mut nodes = self.lock();
        Self::require_parent(&nodes, link)?;
        let displaced = match nodes.get(link) {
            None => Displaced::Nothing,
            Some(Node::Symlink(previous)) => Displaced::Symlink {
                target: previous.clone(),
            },
            Some(node) => {
                let Some(backup) = backup else {
                    return Err(FsError::WouldClobber {
                        path: link.to_path_buf(),
                    });
                };
                let moved = node.clone();
                nodes.insert(backup.to_path_buf(), moved);
                Displaced::BackedUp {
                    backup: backup.to_path_buf(),
                }
            }
        };
        nodes.insert(link.to_path_buf(), Node::Symlink(target.to_path_buf()));
        Ok(displaced)
    }

    fn read_link(&self, path: &Path) -> FsResult<PathBuf> {
        match self.lock().get(path) {
            Some(Node::Symlink(target)) => Ok(target.clone()),
            Some(_) => Err(FsError::NotASymlink {
                path: path.to_path_buf(),
            }),
            None => Err(FsError::NotFound {
                path: path.to_path_buf(),
            }),
        }
    }

    fn remove_symlink(&self, path: &Path) -> FsResult<()> {
        let mut nodes = self.lock();
        match nodes.get(path) {
            Some(Node::Symlink(_)) => {
                nodes.remove(path);
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
        let mut nodes = self.lock();
        if matches!(nodes.get(path), Some(Node::Directory)) {
            return Ok(false);
        }
        let mut dir = Some(path);
        while let Some(d) = dir {
            nodes.entry(d.to_path_buf()).or_insert(Node::Directory);
            dir = d.parent();
        }
        Ok(true)
    }

    fn read_dir(&self, path: &Path) -> FsResult<Vec<PathBuf>> {
        let nodes = self.lock();
        if !matches!(nodes.get(path), Some(Node::Directory)) {
            return Err(FsError::NotFound {
                path: path.to_path_buf(),
            });
        }
        Ok(nodes
            .keys()
            .filter(|p| p.parent() == Some(path))
            .cloned()
            .collect())
    }

    fn rename(&self, from: &Path, to: &Path) -> FsResult<()> {
        let mut nodes = self.lock();
        let Some(node) = nodes.remove(from) else {
            return Err(FsError::NotFound {
                path: from.to_path_buf(),
            });
        };
        Self::require_parent(&nodes, to)?;
        // A directory takes everything under it, which is what `fs::rename`
        // does and what the module cache depends on when it moves a staged
        // extraction into place; see [R-MODULE-063].
        let moved: Vec<PathBuf> = nodes
            .keys()
            .filter(|held| held.starts_with(from))
            .cloned()
            .collect();
        for held in moved {
            let rest = held.strip_prefix(from).unwrap_or(&held).to_path_buf();
            if let Some(child) = nodes.remove(&held) {
                nodes.insert(to.join(rest), child);
            }
        }
        nodes.insert(to.to_path_buf(), node);
        Ok(())
    }

    fn set_executable(&self, path: &Path, executable: bool) -> FsResult<()> {
        match self.lock().get_mut(path) {
            Some(Node::File {
                executable: bit, ..
            }) => {
                // A platform with no mode bits cannot carry one, and an
                // in-memory filesystem that carried it anyway would let a test
                // pass on Windows for behaviour `RealFs` does not have there;
                // see [R-FS-005].
                *bit = executable && cfg!(unix);
                Ok(())
            }
            _ => Err(FsError::NotFound {
                path: path.to_path_buf(),
            }),
        }
    }

    fn entry(&self, path: &Path) -> FsResult<Option<Entry>> {
        Ok(self.lock().get(path).map(|node| match node {
            Node::File { bytes, executable } => Entry::File {
                len: bytes.len() as u64,
                executable: *executable,
            },
            Node::Directory => Entry::Directory,
            Node::Symlink(target) => Entry::Symlink {
                target: target.clone(),
            },
        }))
    }
}
