//! The closed set of reversible effects.

use std::fmt;
use std::path::{Path, PathBuf};

use meowctl_common::Event;
use meowctl_exec::{Command, EventSink, Executor};
use meowctl_fs::{Displaced, Entry, FileSystem};

use crate::{OpsError, OpsResult};

/// The journal's name for an operation.
///
/// These strings are what `internal/rollback/rollback.go` writes, and a
/// journal left by one binary has to replay under the other, so they are
/// fixed; see [R-OPS-020].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpKind {
    /// Replacing a file's contents.
    WriteFile,
    /// Adding a marked block to a file.
    AppendFile,
    /// Copying a file.
    CopyFile,
    /// Creating a symlink.
    Symlink,
    /// Linking a file into place, moving aside what was there.
    LinkFile,
    /// Creating a directory.
    Mkdir,
    /// Writing a downloaded file.
    Download,
    /// Setting a macOS default.
    DefaultsWrite,
    /// Setting a value in a property list.
    PlistSet,
    /// Removing a file. Not a kind `v0.1.0` journals, and never written to a
    /// journal; it exists so an inverse can be expressed as an `Op`.
    Remove,
    /// Removing a directory, for the same reason.
    RemoveDir,
    /// Putting a backed-up file back where it was. An inverse only.
    RestoreBackup,
    /// Doing nothing, for an inverse that has nothing to undo.
    Nothing,
}

impl OpKind {
    /// The name in a journal record.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            OpKind::WriteFile => "write_file",
            OpKind::AppendFile => "append_file",
            OpKind::CopyFile => "copy_file",
            OpKind::Symlink => "symlink",
            OpKind::LinkFile => "link_file",
            OpKind::Mkdir => "mkdir",
            OpKind::Download => "download",
            OpKind::DefaultsWrite => "defaults_write",
            OpKind::PlistSet => "plist_set",
            OpKind::Remove => "remove",
            OpKind::RemoveDir => "remove_dir",
            OpKind::RestoreBackup => "restore_backup",
            OpKind::Nothing => "nothing",
        }
    }

    /// Whether this kind appears in a journal.
    ///
    /// The three that do not are inverses: a journal records the forward
    /// operation and the data to undo it, not the undo itself.
    #[must_use]
    pub const fn is_journaled(self) -> bool {
        !matches!(
            self,
            OpKind::Remove | OpKind::RemoveDir | OpKind::RestoreBackup | OpKind::Nothing
        )
    }
}

impl fmt::Display for OpKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One reversible effect.
///
/// Adding a variant means the compiler asks for its inverse, which is the
/// whole reason this is an enum rather than a set of methods; see [R-OPS-001].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Op {
    /// Replace a file's contents.
    WriteFile {
        /// The file.
        path: PathBuf,
        /// What to write.
        contents: Vec<u8>,
    },

    /// Append a block, wrapped in markers so the inverse can find it again.
    ///
    /// Truncating instead would lose an edit made between apply and undo, and
    /// a component re-running with the same marker replaces its own block
    /// rather than appending a second copy; see [R-OPS-011] and [R-CTX-028].
    AppendFile {
        /// The file.
        path: PathBuf,
        /// What to append, without the markers.
        contents: String,
        /// The identifier in the begin and end markers.
        marker: String,
    },

    /// Copy a file.
    CopyFile {
        /// The source.
        from: PathBuf,
        /// The destination.
        to: PathBuf,
    },

    /// Create a symlink, replacing an existing one.
    Symlink {
        /// What the link points at.
        target: PathBuf,
        /// Where the link goes.
        link: PathBuf,
    },

    /// Link a file into place, moving aside a regular file that is there.
    LinkFile {
        /// What the link points at.
        target: PathBuf,
        /// Where the link goes.
        link: PathBuf,
        /// Where to move an existing regular file.
        backup: PathBuf,
    },

    /// Create a directory and its parents.
    Mkdir {
        /// The directory.
        path: PathBuf,
    },

    /// Write a file that was downloaded.
    ///
    /// Distinct from [`Op::WriteFile`] only in the journal, where the kind is
    /// what `v0.1.0` records. The fetch happens before this: an operation that
    /// reached the network could not be applied against an in-memory
    /// filesystem, and would make every test need a server.
    Download {
        /// Where it goes.
        path: PathBuf,
        /// What was fetched.
        contents: Vec<u8>,
    },

    /// Set a macOS default.
    DefaultsWrite {
        /// The domain.
        domain: String,
        /// The key.
        key: String,
        /// The value, as `defaults` spells it.
        value: String,
        /// The type flag, such as `-bool` or `-int`.
        value_type: String,
    },

    /// Set a value in a property list.
    PlistSet {
        /// The file.
        path: PathBuf,
        /// The key path.
        key: String,
        /// The value.
        value: String,
    },

    /// Remove a file or a symlink. An inverse only.
    Remove {
        /// The path.
        path: PathBuf,
    },

    /// Remove the directories a `mkdir` created. An inverse only.
    ///
    /// `mkdir -p` creates a chain, and undoing only its last link leaves the
    /// rest behind. `inverseMkdir` records one path and removes one
    /// directory, so `~/.config/nvim` undone leaves `~/.config` where there
    /// was nothing. Here the inverse knows where the chain started.
    RemoveDir {
        /// The deepest directory to remove.
        path: PathBuf,
        /// The first ancestor that already existed, which is kept.
        until: PathBuf,
    },

    /// Put a backed-up file back where it was. An inverse only.
    ///
    /// A move rather than a copy: `applyInverseLinkFile` renames, and a copy
    /// leaves the backup behind, so the tree does not come back to where it
    /// started. A property test found exactly that.
    RestoreBackup {
        /// Where the original was moved to.
        backup: PathBuf,
        /// Where it belongs.
        link: PathBuf,
    },

    /// Do nothing. The inverse of an operation that changed nothing.
    Nothing,
}

impl Op {
    /// This operation's journal name.
    #[must_use]
    pub const fn kind(&self) -> OpKind {
        match self {
            Op::WriteFile { .. } => OpKind::WriteFile,
            Op::AppendFile { .. } => OpKind::AppendFile,
            Op::CopyFile { .. } => OpKind::CopyFile,
            Op::Symlink { .. } => OpKind::Symlink,
            Op::LinkFile { .. } => OpKind::LinkFile,
            Op::Mkdir { .. } => OpKind::Mkdir,
            Op::Download { .. } => OpKind::Download,
            Op::DefaultsWrite { .. } => OpKind::DefaultsWrite,
            Op::PlistSet { .. } => OpKind::PlistSet,
            Op::Remove { .. } => OpKind::Remove,
            Op::RemoveDir { .. } => OpKind::RemoveDir,
            Op::RestoreBackup { .. } => OpKind::RestoreBackup,
            Op::Nothing => OpKind::Nothing,
        }
    }

    /// What this operation acts on, for an event and for a message.
    #[must_use]
    pub fn target(&self) -> String {
        match self {
            Op::WriteFile { path, .. }
            | Op::Mkdir { path }
            | Op::Download { path, .. }
            | Op::Remove { path }
            | Op::RemoveDir { path, .. }
            | Op::AppendFile { path, .. } => path.display().to_string(),
            Op::CopyFile { to, .. } => to.display().to_string(),
            Op::Symlink { link, .. }
            | Op::LinkFile { link, .. }
            | Op::RestoreBackup { link, .. } => link.display().to_string(),
            Op::PlistSet { path, key, .. } => format!("{} {key}", path.display()),
            Op::DefaultsWrite { domain, key, .. } => format!("{domain} {key}"),
            Op::Nothing => String::new(),
        }
    }

    /// The operation that undoes this one.
    ///
    /// Computed before applying, because it depends on the state the effect is
    /// about to destroy: the prior content of a file, the prior target of a
    /// symlink, whether a directory already existed; see [R-OPS-003].
    ///
    /// # Errors
    ///
    /// Whatever reading the prior state fails with.
    pub fn inverse(&self, fs: &dyn FileSystem, exec: &dyn Executor) -> OpsResult<Op> {
        Ok(match self {
            // Restoring prior content when the file existed, deleting when it
            // did not. Always deleting would destroy the user's original; see
            // [R-OPS-010].
            Op::WriteFile { path, .. } | Op::Download { path, .. } => match fs.read(path) {
                Ok(prior) => Op::WriteFile {
                    path: path.clone(),
                    contents: prior,
                },
                Err(meowctl_fs::FsError::NotFound { .. }) => Op::Remove { path: path.clone() },
                Err(e) => return Err(e.into()),
            },

            // The marker is how the block is found again, so the inverse
            // carries it rather than a length.
            //
            // Appending to a file that is not there creates it, and removing
            // the block afterwards would leave a zero-byte file where there
            // was none. `applyInverseAppendFile` does exactly that; undoing it
            // properly means removing the file, which is what [R-OPS-004]
            // asks for and what a replayed v0.1.0 record cannot express.
            Op::AppendFile { path, marker, .. } => {
                if fs.entry(path)?.is_some() {
                    Op::AppendFile {
                        path: path.clone(),
                        contents: String::new(),
                        marker: marker.clone(),
                    }
                } else {
                    Op::Remove { path: path.clone() }
                }
            }

            Op::CopyFile { to, .. } => Op::Remove { path: to.clone() },

            Op::Symlink { link, .. } => match fs.entry(link)? {
                Some(Entry::Symlink { target }) => Op::Symlink {
                    target,
                    link: link.clone(),
                },
                Some(_) | None => Op::Remove { path: link.clone() },
            },

            // The backup is the user's original file. Not restoring it is data
            // loss; see [R-OPS-015].
            Op::LinkFile { link, backup, .. } => match fs.entry(link)? {
                Some(Entry::Symlink { target }) => Op::Symlink {
                    target,
                    link: link.clone(),
                },
                Some(_) => Op::RestoreBackup {
                    backup: backup.clone(),
                    link: link.clone(),
                },
                None => Op::Remove { path: link.clone() },
            },

            // Removing a directory meowctl did not create would delete
            // `~/.config` because a component put a file in it; see
            // [R-OPS-013].
            Op::Mkdir { path } => {
                if fs.entry(path)?.is_some() {
                    Op::Nothing
                } else {
                    // Walk up to the first ancestor that is already there;
                    // everything below it is about to be created.
                    let mut until = path.clone();
                    while let Some(parent) = until.parent() {
                        if fs.entry(parent)?.is_some() {
                            break;
                        }
                        until = parent.to_path_buf();
                    }
                    Op::RemoveDir {
                        path: path.clone(),
                        until: until.parent().unwrap_or(&until).to_path_buf(),
                    }
                }
            }

            Op::DefaultsWrite {
                domain,
                key,
                value_type,
                ..
            } => {
                match read_default(exec, domain, key)? {
                    Some(prior) => Op::DefaultsWrite {
                        domain: domain.clone(),
                        key: key.clone(),
                        value: prior,
                        value_type: value_type.clone(),
                    },
                    // No prior value: delete the key rather than write an
                    // empty one, which would leave the machine in a state it
                    // was never in; see [R-OPS-017].
                    None => Op::DefaultsWrite {
                        domain: domain.clone(),
                        key: key.clone(),
                        value: String::new(),
                        value_type: "-delete".to_owned(),
                    },
                }
            }

            Op::PlistSet { path, key, .. } => Op::PlistSet {
                path: path.clone(),
                key: key.clone(),
                value: read_plist(exec, path, key)?.unwrap_or_default(),
            },

            Op::Remove { .. } | Op::RemoveDir { .. } | Op::RestoreBackup { .. } | Op::Nothing => {
                Op::Nothing
            }
        })
    }

    /// Performs the effect.
    ///
    /// Everything goes through the [`FileSystem`] and the [`Executor`], so a
    /// dry run and an in-memory test both work; see [R-OPS-002].
    ///
    /// # Errors
    ///
    /// Whatever the filesystem or the command fails with.
    pub fn apply(
        &self,
        fs: &dyn FileSystem,
        exec: &dyn Executor,
        events: EventSink<'_>,
    ) -> OpsResult<()> {
        match self {
            Op::WriteFile { path, contents } | Op::Download { path, contents } => {
                fs.write(path, contents)?;
            }

            Op::AppendFile {
                path,
                contents,
                marker,
            } => {
                let existing = match fs.read(path) {
                    Ok(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    Err(meowctl_fs::FsError::NotFound { .. }) => String::new(),
                    Err(e) => return Err(e.into()),
                };
                let without = remove_marked_block(&existing, marker);
                let next = if contents.is_empty() {
                    // An empty append is the inverse: the block goes and
                    // nothing replaces it.
                    without
                } else {
                    format!("{without}{}", marked_block(contents, marker))
                };
                fs.write(path, next.as_bytes())?;
            }

            Op::CopyFile { from, to } => fs.copy(from, to)?,

            Op::Symlink { target, link } => {
                fs.symlink(target, link, None)?;
            }

            Op::LinkFile {
                target,
                link,
                backup,
            } => match fs.symlink(target, link, Some(backup))? {
                Displaced::BackedUp { backup } => events(Event::Message {
                    level: meowctl_common::Level::Info,
                    text: format!("moved aside to {}", backup.display()),
                }),
                Displaced::Nothing | Displaced::Symlink { .. } => {}
            },

            Op::Mkdir { path } => {
                fs.create_dir_all(path)?;
            }

            Op::DefaultsWrite {
                domain,
                key,
                value,
                value_type,
            } => {
                let command = if value_type == "-delete" {
                    Command::new("defaults").args(["delete", domain, key])
                } else {
                    Command::new("defaults").args(["write", domain, key, value_type, value])
                };
                run_expecting_success(exec, &command, events)?;
            }

            Op::PlistSet { path, key, value } => {
                let command = Command::new("plutil").args([
                    "-replace",
                    key,
                    "-string",
                    value,
                    &path.display().to_string(),
                ]);
                run_expecting_success(exec, &command, events)?;
            }

            Op::Remove { path } => fs.remove(path)?,

            // Remove the link, then move the original back: the two steps
            // `applyInverseLinkFile` takes, in that order.
            Op::RestoreBackup { backup, link } => {
                fs.remove(link)?;
                fs.rename(backup, link)?;
            }

            // Deepest first, stopping at the ancestor that was already
            // there. A directory the user filled since is left alone: the
            // inverse undoes meowctl's effect, not the user's.
            Op::RemoveDir { path, until } => {
                let mut current = path.clone();
                loop {
                    if current == *until || !fs.read_dir(&current).is_ok_and(|e| e.is_empty()) {
                        break;
                    }
                    fs.remove(&current)?;
                    match current.parent() {
                        Some(parent) => current = parent.to_path_buf(),
                        None => break,
                    }
                }
            }

            Op::Nothing => {}
        }

        if self.kind() != OpKind::Nothing {
            events(Event::OpApplied {
                kind: self.kind().as_str().to_owned(),
                target: self.target(),
            });
        }
        Ok(())
    }
}

/// Wraps a block in the markers the inverse looks for.
fn marked_block(contents: &str, marker: &str) -> String {
    let body = contents.strip_suffix('\n').unwrap_or(contents);
    format!("# BEGIN meowctl {marker}\n{body}\n# END meowctl {marker}\n")
}

/// Removes a marked block, leaving everything around it alone.
///
/// A file edited between apply and undo keeps the edit, which is why the
/// inverse is not a truncation.
fn remove_marked_block(text: &str, marker: &str) -> String {
    let begin = format!("# BEGIN meowctl {marker}");
    let end = format!("# END meowctl {marker}");
    let mut out = String::with_capacity(text.len());
    let mut inside = false;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end_matches(['\n', '\r']);
        if trimmed == begin {
            inside = true;
            continue;
        }
        if trimmed == end {
            inside = false;
            continue;
        }
        if !inside {
            out.push_str(line);
        }
    }
    out
}

/// Reads a macOS default, or `None` when the key is not set.
fn read_default(exec: &dyn Executor, domain: &str, key: &str) -> OpsResult<Option<String>> {
    let mut discard = |_: Event| {};
    let out = exec.run(
        &Command::new("defaults").args(["read", domain, key]),
        &mut discard,
    )?;
    Ok(out.succeeded().then(|| out.stdout.trim_end().to_owned()))
}

/// Reads a value from a property list, or `None` when it is absent.
fn read_plist(exec: &dyn Executor, path: &Path, key: &str) -> OpsResult<Option<String>> {
    let mut discard = |_: Event| {};
    let out = exec.run(
        &Command::new("plutil").args(["-extract", key, "raw", &path.display().to_string()]),
        &mut discard,
    )?;
    Ok(out.succeeded().then(|| out.stdout.trim_end().to_owned()))
}

/// Runs a command that has to succeed for the operation to have happened.
fn run_expecting_success(
    exec: &dyn Executor,
    command: &Command,
    events: EventSink<'_>,
) -> OpsResult<()> {
    let out = exec.run(command, events)?;
    if out.succeeded() {
        return Ok(());
    }
    let mut line = command.program.clone();
    for arg in &command.args {
        line.push(' ');
        line.push_str(arg);
    }
    Err(OpsError::CommandFailed {
        command: line,
        code: out.exit_code.unwrap_or(-1),
        stderr: out.stderr,
    })
}
