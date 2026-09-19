//! What a filesystem operation can fail with.

use std::io;
use std::path::{Path, PathBuf};

use meowctl_common::Severity;

/// A filesystem operation that did not succeed.
///
/// Every variant names the path it was working on. A caller that surfaces
/// "permission denied" without saying which file sends the user reading hook
/// sources to find out; see [R-FS-030].
#[derive(Debug, thiserror::Error)]
pub enum FsError {
    /// The path does not exist.
    ///
    /// Distinct from the other failures because callers act on it: a missing
    /// lock file is a clean slate and a missing `init.star` is fatal; see
    /// [R-FS-032].
    #[error("{path} does not exist")]
    NotFound {
        /// The path that was looked for.
        path: PathBuf,
    },

    /// A symlink was expected and something else was there.
    #[error("{path} is not a symlink")]
    NotASymlink {
        /// The path that was checked.
        path: PathBuf,
    },

    /// A symlink would replace a regular file and no backup was asked for.
    #[error("{path} is a regular file; pass a backup to replace it with a symlink")]
    WouldClobber {
        /// The path that is in the way.
        path: PathBuf,
    },

    /// The parent directory does not exist.
    ///
    /// Separate from `NotFound` so a dry run can predict it: it is the failure
    /// a plan can see without performing the write; see [R-FS-033].
    #[error("{path} cannot be written: its directory {parent} does not exist")]
    NoParent {
        /// The path that was to be written.
        path: PathBuf,
        /// The directory that is missing.
        parent: PathBuf,
    },

    /// Anything the operating system refused.
    #[error("{operation} {path}: {source}")]
    Io {
        /// What was being attempted, as a verb phrase.
        operation: &'static str,
        /// The path it was attempted on.
        path: PathBuf,
        /// The underlying failure.
        source: io::Error,
    },
}

impl FsError {
    /// Builds an [`FsError::Io`], mapping a not-found error to
    /// [`FsError::NotFound`] so callers can tell the two apart.
    pub(crate) fn io(operation: &'static str, path: &Path, source: io::Error) -> Self {
        if source.kind() == io::ErrorKind::NotFound {
            return FsError::NotFound {
                path: path.to_path_buf(),
            };
        }
        FsError::Io {
            operation,
            path: path.to_path_buf(),
            source,
        }
    }

    /// The path the operation was working on.
    #[must_use]
    pub fn path(&self) -> &Path {
        match self {
            FsError::NotFound { path }
            | FsError::NotASymlink { path }
            | FsError::WouldClobber { path }
            | FsError::NoParent { path, .. }
            | FsError::Io { path, .. } => path,
        }
    }

    /// How this classifies for the exit code.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        Severity::General
    }
}

/// The result of a filesystem operation.
pub type FsResult<T> = Result<T, FsError>;
