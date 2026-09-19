//! Every filesystem effect meowctl performs, behind one trait.
//!
//! Nothing above this crate calls `std::fs`. That is what makes a dry run a
//! different implementation rather than a branch repeated in every effectful
//! function, which is how `v0.1.0` did it and how it shipped a dry run that
//! claimed work the runner skipped.
//!
//! Three implementations: [`RealFs`] performs the effect, [`DryRunFs`] records
//! the intent and predicts what a real run would do, and [`MemFs`] holds the
//! tree in memory so a test needs no directory to clean up.
//!
//! Paths arrive already resolved. Expanding `~` or joining against a working
//! directory happens in `meowctl_common::paths`, because a filesystem that
//! resolved its own paths could plan against different files than it writes;
//! see [R-FS-002].

mod dry_run;
mod error;
mod mem;
mod real;

use std::fmt::Debug;
use std::path::{Path, PathBuf};

pub use dry_run::{DryRunFs, Intent};
pub use error::{FsError, FsResult};
pub use mem::MemFs;
pub use real::RealFs;

/// What a path is, when something is there.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Entry {
    /// A regular file, with its length in bytes.
    File {
        /// Length in bytes.
        len: u64,
    },
    /// A directory.
    Directory,
    /// A symbolic link, with the path it points at.
    Symlink {
        /// The link's target, as written.
        target: PathBuf,
    },
}

impl Entry {
    /// Whether this is a symlink.
    #[must_use]
    pub const fn is_symlink(&self) -> bool {
        matches!(self, Entry::Symlink { .. })
    }

    /// Whether this is a directory.
    #[must_use]
    pub const fn is_dir(&self) -> bool {
        matches!(self, Entry::Directory)
    }
}

/// What replacing an existing symlink displaced, so the caller can journal an
/// inverse that puts it back; see [R-OPS-014] and [R-OPS-015].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Displaced {
    /// Nothing was there.
    Nothing,
    /// A symlink was there, pointing at this.
    Symlink {
        /// Where the previous link pointed.
        target: PathBuf,
    },
    /// A regular file was there and was moved aside.
    BackedUp {
        /// Where the original now is.
        backup: PathBuf,
    },
}

/// The filesystem effects meowctl performs.
///
/// Implementations are interchangeable behind a trait object, and
/// `meowctl-cli` picks one from the flags; see [R-FS-014]. Nothing below it
/// chooses, which is what makes `--dry-run` a guarantee rather than a
/// convention.
pub trait FileSystem: Debug {
    /// Reads a file's bytes.
    ///
    /// # Errors
    ///
    /// [`FsError::NotFound`] when nothing is there, or [`FsError::Io`].
    fn read(&self, path: &Path) -> FsResult<Vec<u8>>;

    /// Replaces a file's contents.
    ///
    /// Atomic: the implementation writes a sibling temporary file and renames
    /// it, so a reader never sees a partial file and a crash leaves the
    /// previous contents; see [R-FS-003].
    ///
    /// # Errors
    ///
    /// [`FsError::NoParent`] when the directory does not exist, or
    /// [`FsError::Io`].
    fn write(&self, path: &Path, contents: &[u8]) -> FsResult<()>;

    /// Appends to a file, creating it when absent.
    ///
    /// # Errors
    ///
    /// [`FsError::NoParent`] when the directory does not exist, or
    /// [`FsError::Io`].
    fn append(&self, path: &Path, contents: &[u8]) -> FsResult<()>;

    /// Removes a file or a symlink.
    ///
    /// Removing something that is not there succeeds, because the caller
    /// wanted it gone and it is.
    ///
    /// # Errors
    ///
    /// [`FsError::Io`] when it is there and cannot be removed.
    fn remove(&self, path: &Path) -> FsResult<()>;

    /// Copies a file.
    ///
    /// # Errors
    ///
    /// [`FsError::NotFound`] when the source is missing, [`FsError::NoParent`]
    /// when the destination's directory does not exist, or [`FsError::Io`].
    fn copy(&self, from: &Path, to: &Path) -> FsResult<()>;

    /// Creates a symlink at `link` pointing at `target`, replacing an existing
    /// symlink and reporting what it displaced.
    ///
    /// A regular file at `link` is refused unless `backup` says where to move
    /// it, because a user's own file must not disappear without a record; see
    /// [R-FS-020] and [R-FS-021].
    ///
    /// # Errors
    ///
    /// [`FsError::WouldClobber`] when a regular file is in the way and no
    /// backup was given, or [`FsError::Io`].
    fn symlink(&self, target: &Path, link: &Path, backup: Option<&Path>) -> FsResult<Displaced>;

    /// Reads a symlink's target.
    ///
    /// # Errors
    ///
    /// [`FsError::NotASymlink`] when the path is something else,
    /// [`FsError::NotFound`] when nothing is there, or [`FsError::Io`].
    fn read_link(&self, path: &Path) -> FsResult<PathBuf>;

    /// Removes a symlink, and only a symlink.
    ///
    /// # Errors
    ///
    /// [`FsError::NotASymlink`] when the path is a file or a directory, which
    /// stops a mistyped path from deleting something real; see [R-FS-022].
    fn remove_symlink(&self, path: &Path) -> FsResult<()>;

    /// Creates a directory and its parents.
    ///
    /// Reports whether it created anything, so the caller can journal an
    /// inverse that removes only what meowctl made; see [R-OPS-013].
    ///
    /// # Errors
    ///
    /// [`FsError::Io`] when the directory cannot be created.
    fn create_dir_all(&self, path: &Path) -> FsResult<bool>;

    /// Lists a directory's entries, sorted by name.
    ///
    /// # Errors
    ///
    /// [`FsError::NotFound`] when the directory is missing, or
    /// [`FsError::Io`].
    fn read_dir(&self, path: &Path) -> FsResult<Vec<PathBuf>>;

    /// Renames a path.
    ///
    /// # Errors
    ///
    /// [`FsError::NotFound`] when the source is missing, or [`FsError::Io`].
    fn rename(&self, from: &Path, to: &Path) -> FsResult<()>;

    /// What is at a path, without following a symlink.
    ///
    /// # Errors
    ///
    /// [`FsError::Io`] when the path cannot be inspected. A path that is not
    /// there is `Ok(None)` rather than an error, because asking is how a
    /// component tests before it reads; see [R-CTX-042].
    fn entry(&self, path: &Path) -> FsResult<Option<Entry>>;

    /// Whether anything is at a path.
    ///
    /// # Errors
    ///
    /// [`FsError::Io`] when the path cannot be inspected.
    fn exists(&self, path: &Path) -> FsResult<bool> {
        Ok(self.entry(path)?.is_some())
    }
}

/// Sharing a filesystem is ordinary: the engine hands the same one to every
/// component, and a test keeps a handle to assert on what it holds. The
/// forwarding is mechanical, so it is written once here rather than at each
/// call site through a wrapper type.
impl<T: FileSystem + ?Sized> FileSystem for std::sync::Arc<T> {
    fn read(&self, path: &Path) -> FsResult<Vec<u8>> {
        (**self).read(path)
    }
    fn write(&self, path: &Path, contents: &[u8]) -> FsResult<()> {
        (**self).write(path, contents)
    }
    fn append(&self, path: &Path, contents: &[u8]) -> FsResult<()> {
        (**self).append(path, contents)
    }
    fn remove(&self, path: &Path) -> FsResult<()> {
        (**self).remove(path)
    }
    fn copy(&self, from: &Path, to: &Path) -> FsResult<()> {
        (**self).copy(from, to)
    }
    fn symlink(&self, target: &Path, link: &Path, backup: Option<&Path>) -> FsResult<Displaced> {
        (**self).symlink(target, link, backup)
    }
    fn read_link(&self, path: &Path) -> FsResult<PathBuf> {
        (**self).read_link(path)
    }
    fn remove_symlink(&self, path: &Path) -> FsResult<()> {
        (**self).remove_symlink(path)
    }
    fn create_dir_all(&self, path: &Path) -> FsResult<bool> {
        (**self).create_dir_all(path)
    }
    fn read_dir(&self, path: &Path) -> FsResult<Vec<PathBuf>> {
        (**self).read_dir(path)
    }
    fn rename(&self, from: &Path, to: &Path) -> FsResult<()> {
        (**self).rename(from, to)
    }
    fn entry(&self, path: &Path) -> FsResult<Option<Entry>> {
        (**self).entry(path)
    }
}

/// Reads a file as UTF-8 text.
///
/// A convenience over [`FileSystem::read`], here rather than on the trait so
/// an implementation has one fewer method to get right.
///
/// # Errors
///
/// Whatever [`FileSystem::read`] returns, or [`FsError::Io`] when the bytes
/// are not UTF-8.
pub fn read_to_string(fs: &dyn FileSystem, path: &Path) -> FsResult<String> {
    let bytes = fs.read(path)?;
    String::from_utf8(bytes).map_err(|e| FsError::Io {
        operation: "decoding",
        path: path.to_path_buf(),
        source: std::io::Error::new(std::io::ErrorKind::InvalidData, e),
    })
}
