//! Reading a module tarball into a directory.
//!
//! Every write goes through [`FileSystem`], so an extraction is testable
//! against `MemFs` and invisible to the real disk under a dry run. The archive
//! is remote input, so every entry's path is checked before anything is
//! written; see [R-MODULE-033].

use std::io::Read as _;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;
use meowctl_fs::FileSystem;

use crate::{ModuleError, ModuleResult};

/// One file from an archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchiveFile {
    /// The path inside the module, with `/` separators.
    pub path: String,
    /// The contents.
    pub contents: Vec<u8>,
    /// Whether the entry carried any executable bit; see [R-MODULE-032].
    pub executable: bool,
}

/// Reads a gzipped tar into the files it holds, in archive order.
///
/// Directories, symlinks, hard links, and device entries are dropped, which is
/// what `extractTarEntry` does: a module is a tree of files, and a symlink
/// inside one is a way out of it that the containment check cannot see.
///
/// # Errors
///
/// [`ModuleError::Archive`] when the bytes are not a gzipped tar, or when an
/// entry's path escapes the module root.
pub fn read(module: &str, data: &[u8]) -> ModuleResult<Vec<ArchiveFile>> {
    let bad = |reason: String| ModuleError::Archive {
        module: module.to_owned(),
        reason,
    };

    let mut archive = tar::Archive::new(GzDecoder::new(data));
    let entries = archive
        .entries()
        .map_err(|e| bad(format!("the archive could not be read: {e}")))?;

    let mut files = Vec::new();
    for entry in entries {
        let mut entry = entry.map_err(|e| bad(format!("an entry could not be read: {e}")))?;
        if entry.header().entry_type() != tar::EntryType::Regular {
            continue;
        }
        let raw = entry
            .path()
            .map_err(|e| bad(format!("an entry has an unusable path: {e}")))?
            .into_owned();
        let path = safe_path(&raw)
            .ok_or_else(|| bad(format!("the entry {} is outside the module", raw.display())))?;
        let executable = entry.header().mode().is_ok_and(|mode| mode & 0o111 != 0);
        let mut contents = Vec::new();
        entry
            .read_to_end(&mut contents)
            .map_err(|e| bad(format!("the entry {path} could not be read: {e}")))?;
        files.push(ArchiveFile {
            path,
            contents,
            executable,
        });
    }
    Ok(files)
}

/// Drops the single top-level directory every entry shares, if there is one.
///
/// A release tarball has `MODULE.meow` at its root and nothing is dropped. A
/// GitHub archive has everything under `repo-<commit>/` and that segment goes;
/// see [R-MODULE-034].
#[must_use]
pub fn strip_single_root(files: Vec<ArchiveFile>) -> Vec<ArchiveFile> {
    let Some(shared) = shared_root(&files) else {
        return files;
    };
    files
        .into_iter()
        .map(|file| ArchiveFile {
            path: file.path[shared.len() + 1..].to_owned(),
            ..file
        })
        .collect()
}

/// The top-level directory every entry is under, when there is exactly one.
fn shared_root(files: &[ArchiveFile]) -> Option<String> {
    let mut shared: Option<&str> = None;
    for file in files {
        let (first, rest) = file.path.split_once('/')?;
        if rest.is_empty() {
            return None;
        }
        match shared {
            None => shared = Some(first),
            Some(seen) if seen == first => {}
            Some(_) => return None,
        }
    }
    shared.map(str::to_owned)
}

/// Writes files under `root`, creating directories as needed.
///
/// # Errors
///
/// Whatever the filesystem returns.
pub fn write_into(fs: &dyn FileSystem, root: &Path, files: &[ArchiveFile]) -> ModuleResult<()> {
    fs.create_dir_all(root)?;
    for file in files {
        let target = root.join(file.path.replace('/', std::path::MAIN_SEPARATOR_STR));
        if let Some(parent) = target.parent() {
            fs.create_dir_all(parent)?;
        }
        fs.write(&target, &file.contents)?;
        if file.executable {
            fs.set_executable(&target, true)?;
        }
    }
    Ok(())
}

/// Normalises an entry path, or refuses it.
///
/// Refuses an absolute path and one that climbs out, which is what
/// `extractTarEntry` refuses, and returns `/`-separated text because that is
/// what a module path is everywhere else in this crate.
fn safe_path(raw: &Path) -> Option<String> {
    if raw.is_absolute() || raw.has_root() {
        return None;
    }
    let mut normal = PathBuf::new();
    for part in raw.components() {
        match part {
            std::path::Component::Normal(p) => normal.push(p),
            std::path::Component::CurDir => {}
            _ => return None,
        }
    }
    if normal.as_os_str().is_empty() {
        return None;
    }
    let text = normal.to_str()?.replace('\\', "/");
    Some(text)
}
