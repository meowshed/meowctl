//! `.hook-error`, the flag a failed runtime hook leaves behind.
//!
//! `meowctl hook shell` runs on every shell spawn, and a shell that cannot
//! start is worse than a shell that starts without its integration. So the
//! command reports nothing and exits zero, and writes what went wrong here
//! instead; `status` and `doctor` are where the user meets it. See
//! [R-CLI-062] and [R-CONFIG-064].

use std::path::Path;

use meowctl_fs::FileSystem;

use crate::ConfigResult;

/// What a failed runtime hook recorded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HookError {
    /// When it failed, as an RFC 3339 timestamp.
    pub at: String,
    /// What went wrong.
    pub reason: String,
}

impl HookError {
    /// Reads the flag, if there is one.
    ///
    /// A file that is there but unreadable is reported as present with an
    /// empty reason rather than as absent: the user's shell has no
    /// integration either way, and saying nothing would hide that.
    #[must_use]
    pub fn read(fs: &dyn FileSystem, path: &Path) -> Option<HookError> {
        let bytes = fs.read(path).ok()?;
        let text = String::from_utf8_lossy(&bytes);
        let mut lines = text.lines();
        Some(HookError {
            at: lines.next().unwrap_or_default().to_owned(),
            reason: lines.collect::<Vec<_>>().join("\n"),
        })
    }

    /// Whether the flag is there at all.
    #[must_use]
    pub fn present(fs: &dyn FileSystem, path: &Path) -> bool {
        matches!(fs.entry(path), Ok(Some(_)))
    }

    /// Writes the flag, replacing whatever was there.
    ///
    /// A flag rather than a log: what a user needs is the reason their shell
    /// has no integration now, not a history of every spawn since it broke;
    /// see [R-CONFIG-064]. The `0o600` that requirement asks for is what
    /// [R-FS-004] gives every created file.
    ///
    /// # Errors
    ///
    /// Whatever the filesystem fails with.
    pub fn write(&self, fs: &dyn FileSystem, path: &Path) -> ConfigResult<()> {
        let body = format!("{}\n{}\n", self.at, self.reason);
        fs.write(path, body.as_bytes())?;
        Ok(())
    }

    /// Removes the flag.
    ///
    /// Absence is the success case, so removing one that is not there is not
    /// an error; see [R-CLI-063].
    ///
    /// # Errors
    ///
    /// Whatever the filesystem fails with, other than the file not existing.
    pub fn clear(fs: &dyn FileSystem, path: &Path) -> ConfigResult<()> {
        match fs.remove(path) {
            Ok(()) | Err(meowctl_fs::FsError::NotFound { .. }) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}
