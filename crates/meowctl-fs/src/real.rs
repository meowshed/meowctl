//! The implementation that performs the effect.

use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::{Displaced, Entry, FileSystem, FsError, FsResult};

/// Mode for a file meowctl creates.
///
/// `0o600` because a configuration holds whatever a user's components put in
/// it, and the narrow default is the safe one; see [R-FS-004].
///
/// Declared for every platform even though only Unix applies it, so that a
/// call site does not need a `cfg` of its own; [`set_mode`] is where the
/// platform difference lives.
const FILE_MODE: u32 = 0o600;

/// Mode for a directory meowctl creates.
const DIR_MODE: u32 = 0o700;

/// Performs filesystem effects against the real filesystem.
#[derive(Debug, Clone, Copy, Default)]
pub struct RealFs;

impl RealFs {
    /// A new one. It holds no state.
    #[must_use]
    pub const fn new() -> Self {
        RealFs
    }
}

/// Checks that a path's parent directory exists, so a write fails with a
/// reason a dry run could also have predicted; see [R-FS-033].
fn require_parent(path: &Path) -> FsResult<&Path> {
    let parent = path.parent().ok_or_else(|| FsError::NoParent {
        path: path.to_path_buf(),
        parent: PathBuf::new(),
    })?;
    if parent.as_os_str().is_empty() || parent.is_dir() {
        return Ok(parent);
    }
    Err(FsError::NoParent {
        path: path.to_path_buf(),
        parent: parent.to_path_buf(),
    })
}

#[cfg(unix)]
fn set_mode(path: &Path, mode: u32) -> FsResult<()> {
    use std::os::unix::fs::PermissionsExt as _;
    fs::set_permissions(path, fs::Permissions::from_mode(mode))
        .map_err(|e| FsError::io("setting the mode of", path, e))
}

#[cfg(not(unix))]
fn set_mode(_path: &Path, _mode: u32) -> FsResult<()> {
    // Windows has no mode bits to set. The narrow default exists to keep a
    // configuration private on a multi-user Unix machine; the equivalent there
    // is an ACL, which meowctl does not manage.
    Ok(())
}

impl FileSystem for RealFs {
    fn read(&self, path: &Path) -> FsResult<Vec<u8>> {
        fs::read(path).map_err(|e| FsError::io("reading", path, e))
    }

    fn write(&self, path: &Path, contents: &[u8]) -> FsResult<()> {
        let parent = require_parent(path)?;

        // A sibling temporary file, so the rename is on one filesystem and is
        // therefore atomic. `internal/lock/write.go` does the same, and the
        // lock files depend on it.
        let temp = temp_sibling(path);
        let mut file = fs::File::create(&temp).map_err(|e| FsError::io("creating", &temp, e))?;

        let written = file
            .write_all(contents)
            .and_then(|()| file.sync_all())
            .map_err(|e| FsError::io("writing", &temp, e));
        drop(file);

        if let Err(e) = written {
            let _ = fs::remove_file(&temp);
            return Err(e);
        }

        if let Err(e) = set_mode(&temp, FILE_MODE) {
            let _ = fs::remove_file(&temp);
            return Err(e);
        }

        fs::rename(&temp, path).map_err(|e| {
            // Leaving the temporary file behind litters the configuration
            // directory, which is what `v0.1.0` guards on every error branch;
            // see [R-FS-031].
            let _ = fs::remove_file(&temp);
            FsError::io("renaming into place", path, e)
        })?;
        let _ = parent;
        Ok(())
    }

    fn append(&self, path: &Path, contents: &[u8]) -> FsResult<()> {
        require_parent(path)?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(|e| FsError::io("opening for append", path, e))?;
        file.write_all(contents)
            .map_err(|e| FsError::io("appending to", path, e))?;
        set_mode(path, FILE_MODE)
    }

    fn remove(&self, path: &Path) -> FsResult<()> {
        match fs::symlink_metadata(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(FsError::io("inspecting", path, e)),
            Ok(_) => fs::remove_file(path).map_err(|e| FsError::io("removing", path, e)),
        }
    }

    fn copy(&self, from: &Path, to: &Path) -> FsResult<()> {
        require_parent(to)?;
        fs::copy(from, to).map_err(|e| FsError::io("copying", from, e))?;
        set_mode(to, FILE_MODE)
    }

    fn symlink(&self, target: &Path, link: &Path, backup: Option<&Path>) -> FsResult<Displaced> {
        require_parent(link)?;
        let displaced = match fs::symlink_metadata(link) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Displaced::Nothing,
            Err(e) => return Err(FsError::io("inspecting", link, e)),
            Ok(meta) if meta.is_symlink() => {
                let previous =
                    fs::read_link(link).map_err(|e| FsError::io("reading the link at", link, e))?;
                fs::remove_file(link).map_err(|e| FsError::io("removing the link at", link, e))?;
                Displaced::Symlink { target: previous }
            }
            Ok(_) => {
                let Some(backup) = backup else {
                    return Err(FsError::WouldClobber {
                        path: link.to_path_buf(),
                    });
                };
                fs::rename(link, backup).map_err(|e| FsError::io("backing up", link, e))?;
                Displaced::BackedUp {
                    backup: backup.to_path_buf(),
                }
            }
        };

        create_symlink(target, link)?;
        Ok(displaced)
    }

    fn read_link(&self, path: &Path) -> FsResult<PathBuf> {
        let meta = fs::symlink_metadata(path).map_err(|e| FsError::io("inspecting", path, e))?;
        if !meta.is_symlink() {
            return Err(FsError::NotASymlink {
                path: path.to_path_buf(),
            });
        }
        fs::read_link(path).map_err(|e| FsError::io("reading the link at", path, e))
    }

    fn remove_symlink(&self, path: &Path) -> FsResult<()> {
        let meta = fs::symlink_metadata(path).map_err(|e| FsError::io("inspecting", path, e))?;
        if !meta.is_symlink() {
            return Err(FsError::NotASymlink {
                path: path.to_path_buf(),
            });
        }
        fs::remove_file(path).map_err(|e| FsError::io("removing the link at", path, e))
    }

    fn create_dir_all(&self, path: &Path) -> FsResult<bool> {
        if path.is_dir() {
            return Ok(false);
        }
        fs::create_dir_all(path).map_err(|e| FsError::io("creating", path, e))?;
        set_mode(path, DIR_MODE)?;
        Ok(true)
    }

    fn read_dir(&self, path: &Path) -> FsResult<Vec<PathBuf>> {
        let mut out = Vec::new();
        for entry in fs::read_dir(path).map_err(|e| FsError::io("listing", path, e))? {
            let entry = entry.map_err(|e| FsError::io("listing", path, e))?;
            out.push(entry.path());
        }
        out.sort();
        Ok(out)
    }

    fn rename(&self, from: &Path, to: &Path) -> FsResult<()> {
        require_parent(to)?;
        fs::rename(from, to).map_err(|e| FsError::io("renaming", from, e))
    }

    fn entry(&self, path: &Path) -> FsResult<Option<Entry>> {
        match fs::symlink_metadata(path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(FsError::io("inspecting", path, e)),
            Ok(meta) if meta.is_symlink() => Ok(Some(Entry::Symlink {
                target: fs::read_link(path)
                    .map_err(|e| FsError::io("reading the link at", path, e))?,
            })),
            Ok(meta) if meta.is_dir() => Ok(Some(Entry::Directory)),
            Ok(meta) => Ok(Some(Entry::File { len: meta.len() })),
        }
    }
}

/// A temporary name beside `path`, so the rename that follows stays on one
/// filesystem.
fn temp_sibling(path: &Path) -> PathBuf {
    let name = path.file_name().map_or_else(
        || std::ffi::OsString::from("meowctl"),
        std::ffi::OsStr::to_os_string,
    );
    let mut temp = std::ffi::OsString::from(".");
    temp.push(&name);
    temp.push(format!(".meowctl-{}.tmp", std::process::id()));
    path.parent().unwrap_or(Path::new(".")).join(temp)
}

#[cfg(unix)]
fn create_symlink(target: &Path, link: &Path) -> FsResult<()> {
    std::os::unix::fs::symlink(target, link).map_err(|e| FsError::io("linking", link, e))
}

#[cfg(not(unix))]
fn create_symlink(target: &Path, link: &Path) -> FsResult<()> {
    // Windows distinguishes a file link from a directory link and needs a
    // privilege for either. meowctl manages dotfiles, which is a Unix
    // workflow; saying so beats creating the wrong kind of link.
    Err(FsError::Io {
        operation: "linking",
        path: link.to_path_buf(),
        source: std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            format!(
                "creating a symlink to {} is not supported on this platform",
                target.display()
            ),
        ),
    })
}
