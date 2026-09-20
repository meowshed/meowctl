//! Where the files are, and what a pre-rename directory looks like.

use std::path::{Path, PathBuf};

use meowctl_fs::FileSystem;

use crate::{ConfigError, ConfigResult};

/// The file `v0.1.0` renamed away from.
///
/// A directory that has it and no `init.star` was written before the rename,
/// and the user is told how to migrate rather than left with a missing-file
/// error; see [R-CONFIG-004].
pub const LEGACY_ENTRY: &str = "meowctl.star";

/// The configuration directory and the files in it.
///
/// The names come from `internal/cli/config.go` and are the ones a user has on
/// disk, so they are fixed; see [R-CONFIG-001].
#[derive(Debug, Clone)]
pub struct Layout {
    root: PathBuf,
}

impl Layout {
    /// A layout rooted at a configuration directory.
    #[must_use]
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Layout { root: root.into() }
    }

    /// The directory itself.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// `init.star`, the user's entry point.
    #[must_use]
    pub fn entry(&self) -> PathBuf {
        self.root.join("init.star")
    }

    /// `local.star`, machine-local and gitignored.
    #[must_use]
    pub fn local_entry(&self) -> PathBuf {
        self.root.join("local.star")
    }

    /// `deps.mod`, the module manifest.
    #[must_use]
    pub fn modfile(&self) -> PathBuf {
        self.root.join("deps.mod")
    }

    /// `deps.local.mod`, machine-local module overrides.
    #[must_use]
    pub fn local_modfile(&self) -> PathBuf {
        self.root.join("deps.local.mod")
    }

    /// `deps.lock`.
    #[must_use]
    pub fn lock(&self) -> PathBuf {
        self.root.join("deps.lock")
    }

    /// `deps.local.lock`.
    #[must_use]
    pub fn local_lock(&self) -> PathBuf {
        self.root.join("deps.local.lock")
    }

    /// `pkgs.lock`.
    #[must_use]
    pub fn packages_lock(&self) -> PathBuf {
        self.root.join("pkgs.lock")
    }

    /// `pkgs.local.lock`.
    #[must_use]
    pub fn local_packages_lock(&self) -> PathBuf {
        self.root.join("pkgs.local.lock")
    }

    /// `state.toml`, the sentinel.
    #[must_use]
    pub fn state(&self) -> PathBuf {
        self.root.join("state.toml")
    }

    /// `installed.lock`.
    #[must_use]
    pub fn installed(&self) -> PathBuf {
        self.root.join("installed.lock")
    }

    /// `rollback.jsonl`, the write-ahead journal.
    #[must_use]
    pub fn journal(&self) -> PathBuf {
        self.root.join("rollback.jsonl")
    }

    /// `theme.toml`, the palette a user can point us at.
    ///
    /// In the configuration directory rather than somewhere of its own,
    /// because a user who moves their configuration between machines expects
    /// their colours to come with it; see [R-TUI-053].
    #[must_use]
    pub fn theme(&self) -> PathBuf {
        self.root.join("theme.toml")
    }

    /// `.hook-error`, the flag a failed runtime hook leaves.
    #[must_use]
    pub fn hook_error(&self) -> PathBuf {
        self.root.join(".hook-error")
    }

    /// The `components/` directory.
    #[must_use]
    pub fn components(&self) -> PathBuf {
        self.root.join("components")
    }

    /// Checks that this directory is one meowctl can use.
    ///
    /// # Errors
    ///
    /// [`ConfigError::LegacyLayout`] when it has the pre-rename layout, or
    /// [`ConfigError::Missing`] when there is no `init.star`.
    pub fn check(&self, fs: &dyn FileSystem) -> ConfigResult<()> {
        let entry = self.entry();
        if fs.exists(&entry)? {
            return Ok(());
        }

        let legacy = self.root.join(LEGACY_ENTRY);
        if fs.exists(&legacy)? {
            // The same three commands `reportLegacyConfig` prints, because a
            // user who has this layout needs the fix rather than the diagnosis.
            return Err(ConfigError::LegacyLayout(format!(
                "found a pre-rename configuration in {}; rename its files to continue:\n  \
                 mv {} {}\n  mv {} {}\n  mv {} {}",
                self.root.display(),
                legacy.display(),
                entry.display(),
                self.root.join("meowctl.mod").display(),
                self.modfile().display(),
                self.root.join("meowctl.lock").display(),
                self.lock().display(),
            )));
        }

        Err(ConfigError::Missing { path: entry })
    }
}
