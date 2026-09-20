//! Why a module did not resolve.

use std::path::PathBuf;

use meowctl_fs::FsError;
use meowctl_net::NetError;
use thiserror::Error;

use crate::MvsError;

/// The result of resolving, fetching, or loading a module.
pub type ModuleResult<T> = Result<T, ModuleError>;

/// Why a module did not resolve.
///
/// Every variant names the module or the URL it is about, because a resolution
/// fetches an index, a manifest, and a tarball, and an error that says only
/// what went wrong does not say where.
#[derive(Debug, Error)]
pub enum ModuleError {
    /// A `load()` argument that is not a module URL.
    #[error("{url} is not a module URL: {reason}")]
    UnusableUrl {
        /// What the caller wrote.
        url: String,
        /// Why it could not be read.
        reason: String,
    },

    /// The registry index could not be fetched.
    ///
    /// [R-MODULE-060] requires saying which failure it was, which is why the
    /// network error is carried rather than flattened to a string.
    #[error("the registry index at {url} could not be fetched: {source}")]
    IndexUnavailable {
        /// Where it was looked for.
        url: String,
        /// What the request did.
        #[source]
        source: NetError,
    },

    /// The registry index was fetched and is not an index.
    #[error("the registry index at {url} could not be read: {reason}")]
    IndexUnreadable {
        /// Where it came from.
        url: String,
        /// What the parser said.
        reason: String,
    },

    /// The index does not list this module; see [R-MODULE-061].
    #[error("the registry has no module named {module}")]
    NoSuchModule {
        /// The name that was looked up.
        module: String,
    },

    /// The index lists the module and not this version; see [R-MODULE-061].
    #[error("the registry has {module}, and no version {version} of it")]
    NoSuchVersion {
        /// The module.
        module: String,
        /// The version that was asked for.
        version: String,
        /// What the index does list, for the message the caller builds.
        available: Vec<String>,
    },

    /// The index lists the module with no versions at all.
    #[error("the registry lists {module} with no versions")]
    NoVersions {
        /// The module.
        module: String,
    },

    /// A fetch failed.
    #[error("fetching {what} for {module}: {source}")]
    Fetch {
        /// The module being fetched.
        module: String,
        /// Which part of it: the tarball, the commit, the file.
        what: String,
        /// What the request did.
        #[source]
        source: NetError,
    },

    /// What was fetched does not hash to what was expected.
    ///
    /// [R-MODULE-031] wants all three of these in the message, because a
    /// mismatch is either corruption or tampering and the reader has to be
    /// able to tell which hash they are looking at.
    #[error("{what} for {module} hashes to {actual}, and {expected} was expected")]
    IntegrityMismatch {
        /// The module.
        module: String,
        /// What was hashed.
        what: String,
        /// What the index or the cache record said.
        expected: String,
        /// What it actually hashes to.
        actual: String,
    },

    /// The archive could not be read, or holds something it may not.
    #[error("the archive for {module} could not be extracted: {reason}")]
    Archive {
        /// The module.
        module: String,
        /// What went wrong.
        reason: String,
    },

    /// A `replace` points at a directory that is not there; see [R-MODULE-064].
    #[error("{module} is replaced by {path}, and there is nothing there")]
    NoSuchReplacement {
        /// The module.
        module: String,
        /// Where it was told to look.
        path: PathBuf,
    },

    /// A file the module should hold is not in it.
    #[error("{module} has no file at {path}")]
    NoSuchFile {
        /// The module.
        module: String,
        /// The path inside it.
        path: String,
    },

    /// Version selection failed.
    #[error(transparent)]
    Selection(#[from] MvsError),

    /// The filesystem refused.
    #[error(transparent)]
    FileSystem(#[from] FsError),
}
