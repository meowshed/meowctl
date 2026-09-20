//! The write-ahead journal, in the format `v0.1.0` writes.
//!
//! One JSON object per line: a sequence number, the phase, the component, the
//! forward operation's kind, and the payload needed to undo it. Two binaries
//! share a configuration directory during the rewrite, so a journal written by
//! one has to replay under the other; the field names here are read from
//! `internal/rollback/rollback.go` and are not ours to change; see
//! [R-OPS-020].

use std::io::{BufRead as _, Write as _};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Op, OpKind, OpsError, OpsResult};

/// One line of the journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    /// Position in the journal, counting from one.
    pub seq: usize,
    /// The phase the operation ran in.
    pub phase: String,
    /// The component that asked for it.
    pub component: String,
    /// The forward operation's kind.
    pub kind: String,
    /// What is needed to undo it.
    pub inverse: serde_json::Value,
}

/// How a replay went.
///
/// Three outcomes rather than a boolean, because `partial` is the one a user
/// most needs to see and it is what `state.toml` already records; see
/// [R-OPS-024] and [R-CONFIG-044].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Every inverse applied.
    Ok,
    /// Some applied and some did not.
    Partial,
    /// None applied.
    Failed,
}

impl Outcome {
    /// The string `state.toml` stores.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Outcome::Ok => "ok",
            Outcome::Partial => "partial",
            Outcome::Failed => "failed",
        }
    }
}

/// What a replay did.
#[derive(Debug)]
pub struct Replay {
    /// How it went overall.
    pub outcome: Outcome,
    /// How many inverses applied.
    pub applied: usize,
    /// The ones that did not, with why.
    pub failures: Vec<(Record, OpsError)>,
}

/// The write-ahead journal.
///
/// Append a record before applying its operation. A crash between the two
/// leaves a journal that replays a no-op, which is safe; the other order
/// leaves an effect with no undo, which is not; see [R-OPS-021].
#[derive(Debug)]
pub struct Journal {
    path: PathBuf,
    seq: usize,
}

impl Journal {
    /// Opens the journal at `path`, continuing its numbering.
    ///
    /// # Errors
    ///
    /// When the file exists and cannot be read.
    pub fn open(path: impl Into<PathBuf>) -> OpsResult<Self> {
        let path = path.into();
        let seq = match std::fs::read_to_string(&path) {
            Ok(text) => text.lines().filter(|l| !l.trim().is_empty()).count(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => 0,
            Err(e) => {
                return Err(OpsError::Journal {
                    kind: "opening",
                    source: e,
                });
            }
        };
        Ok(Journal { path, seq })
    }

    /// Whether the journal holds operations that were never undone.
    ///
    /// A non-empty journal at startup is an interrupted previous run, and the
    /// engine reports it before doing anything else; see [R-ENGINE-042].
    #[must_use]
    pub const fn is_pending(&self) -> bool {
        self.seq > 0
    }

    /// How many records are in it.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.seq
    }

    /// Whether it holds nothing.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.seq == 0
    }

    /// Records how to undo `forward`, before it is applied.
    ///
    /// # Errors
    ///
    /// [`OpsError::Journal`] when the record cannot be written. The caller
    /// must not apply the operation then: an effect with no record of how to
    /// undo it is what this crate exists to prevent; see [R-OPS-032].
    pub fn append(
        &mut self,
        phase: &str,
        component: &str,
        forward: &Op,
        inverse: &Op,
    ) -> OpsResult<()> {
        if !forward.kind().is_journaled() {
            return Ok(());
        }

        self.seq += 1;
        let record = Record {
            seq: self.seq,
            phase: phase.to_owned(),
            component: component.to_owned(),
            kind: forward.kind().as_str().to_owned(),
            inverse: payload(forward, inverse),
        };

        let line = serde_json::to_string(&record).map_err(|e| OpsError::Journal {
            kind: forward.kind().as_str(),
            source: std::io::Error::other(e),
        })?;

        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|e| OpsError::Journal {
                kind: forward.kind().as_str(),
                source: e,
            })?;
        writeln!(file, "{line}")
            .and_then(|()| file.sync_all())
            .map_err(|e| OpsError::Journal {
                kind: forward.kind().as_str(),
                source: e,
            })?;
        Ok(())
    }

    /// Reads every record, in order.
    ///
    /// A line that cannot be parsed is reported and skipped rather than
    /// aborting, because one corrupt line must not strand every earlier
    /// operation; see [R-OPS-030].
    ///
    /// # Errors
    ///
    /// When the journal cannot be read at all.
    pub fn records(&self) -> OpsResult<(Vec<Record>, Vec<OpsError>)> {
        let file = match std::fs::File::open(&self.path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                return Ok((Vec::new(), Vec::new()));
            }
            Err(e) => {
                return Err(OpsError::Journal {
                    kind: "reading",
                    source: e,
                });
            }
        };

        let mut records = Vec::new();
        let mut broken = Vec::new();
        for (index, line) in std::io::BufReader::new(file).lines().enumerate() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    broken.push(OpsError::UnreadableRecord {
                        seq: index + 1,
                        reason: e.to_string(),
                    });
                    continue;
                }
            };
            if line.trim().is_empty() {
                continue;
            }
            match serde_json::from_str::<Record>(&line) {
                Ok(r) => records.push(r),
                Err(e) => broken.push(OpsError::UnreadableRecord {
                    seq: index + 1,
                    reason: e.to_string(),
                }),
            }
        }
        Ok((records, broken))
    }

    /// Removes the journal after a successful run.
    ///
    /// # Errors
    ///
    /// When the file cannot be removed.
    pub fn truncate(&mut self) -> OpsResult<()> {
        match std::fs::remove_file(&self.path) {
            Ok(()) | Err(_) if !self.path.exists() => {
                self.seq = 0;
                Ok(())
            }
            Err(e) => Err(OpsError::Journal {
                kind: "truncating",
                source: e,
            }),
            Ok(()) => {
                self.seq = 0;
                Ok(())
            }
        }
    }
}

/// Rebuilds the inverse operation a record describes.
///
/// # Errors
///
/// [`OpsError::UnreadableRecord`] when the payload does not match its kind.
pub fn inverse_from(record: &Record) -> OpsResult<Op> {
    let bad = |reason: &str| OpsError::UnreadableRecord {
        seq: record.seq,
        reason: reason.to_owned(),
    };
    let text =
        |key: &str| -> Option<String> { record.inverse.get(key)?.as_str().map(str::to_owned) };
    let flag =
        |key: &str| record.inverse.get(key).and_then(serde_json::Value::as_bool) == Some(true);

    Ok(match record.kind.as_str() {
        "write_file" | "download" => {
            let key = if record.kind == "download" {
                "dst"
            } else {
                "path"
            };
            let path = PathBuf::from(text(key).ok_or_else(|| bad("no path"))?);
            if flag("had_prior") {
                Op::WriteFile {
                    path,
                    contents: text("prior_content").unwrap_or_default().into_bytes(),
                }
            } else {
                Op::Remove { path }
            }
        }
        "append_file" => {
            let path = PathBuf::from(text("path").ok_or_else(|| bad("no path"))?);
            if flag("created") {
                Op::Remove { path }
            } else {
                Op::AppendFile {
                    path,
                    contents: String::new(),
                    marker: text("marker").ok_or_else(|| bad("no marker"))?,
                }
            }
        }
        "copy_file" => Op::Remove {
            path: PathBuf::from(text("dst").ok_or_else(|| bad("no dst"))?),
        },
        "symlink" => {
            let link = PathBuf::from(text("dst").ok_or_else(|| bad("no dst"))?);
            if flag("had_prior") {
                Op::Symlink {
                    target: PathBuf::from(text("prior_target").unwrap_or_default()),
                    link,
                }
            } else {
                Op::Remove { path: link }
            }
        }
        "link_file" => {
            let link = PathBuf::from(text("dst").ok_or_else(|| bad("no dst"))?);
            if flag("was_backed_up") {
                Op::RestoreBackup {
                    backup: PathBuf::from(text("backup_path").unwrap_or_default()),
                    link,
                }
            } else {
                Op::Remove { path: link }
            }
        }
        "mkdir" => {
            let path = PathBuf::from(text("path").ok_or_else(|| bad("no path"))?);
            if flag("created_by_meowctl") {
                let until = text("until").map_or_else(
                    // A record from v0.1.0 has no `until`, so undoing removes
                    // the one directory it always removed.
                    || path.parent().unwrap_or(&path).to_path_buf(),
                    PathBuf::from,
                );
                Op::RemoveDir { path, until }
            } else {
                Op::Nothing
            }
        }
        "defaults_write" => Op::DefaultsWrite {
            domain: text("domain").ok_or_else(|| bad("no domain"))?,
            key: text("key").ok_or_else(|| bad("no key"))?,
            value: text("value").unwrap_or_default(),
            value_type: text("value_type").unwrap_or_else(|| "-string".to_owned()),
        },
        "plist_set" => Op::PlistSet {
            path: PathBuf::from(text("path").ok_or_else(|| bad("no path"))?),
            key: text("key").ok_or_else(|| bad("no key"))?,
            value: text("value").unwrap_or_default(),
        },
        other => return Err(bad(&format!("unknown kind {other}"))),
    })
}

/// The payload `v0.1.0` stores for this kind.
///
/// The shapes come from the `inverse*` structs in
/// `internal/rollback/rollback.go`, field names included.
fn payload(forward: &Op, inverse: &Op) -> serde_json::Value {
    use serde_json::json;

    match forward {
        Op::WriteFile { path, .. } => match inverse {
            Op::WriteFile { contents, .. } => json!({
                "path": path,
                "prior_content": String::from_utf8_lossy(contents),
                "had_prior": true,
            }),
            _ => json!({ "path": path, "had_prior": false }),
        },
        Op::Download { path, .. } => match inverse {
            Op::WriteFile { contents, .. } => json!({
                "dst": path,
                "prior_content": String::from_utf8_lossy(contents),
                "had_prior": true,
            }),
            _ => json!({ "dst": path, "had_prior": false }),
        },
        // `created` is ours. v0.1.0 ignores an unknown field, so a record
        // written here still replays there — it removes the block and leaves
        // the empty file, which is what it does today anyway.
        Op::AppendFile { path, marker, .. } => json!({
            "path": path,
            "marker": marker,
            "created": matches!(inverse, Op::Remove { .. }),
        }),
        Op::CopyFile { to, .. } => json!({ "dst": to }),
        Op::Symlink { link, .. } => match inverse {
            Op::Symlink { target, .. } => json!({
                "dst": link,
                "prior_target": target,
                "had_prior": true,
            }),
            _ => json!({ "dst": link, "had_prior": false }),
        },
        Op::LinkFile { link, backup, .. } => match inverse {
            Op::RestoreBackup { .. } => json!({
                "dst": link,
                "backup_path": backup,
                "was_backed_up": true,
            }),
            _ => json!({ "dst": link, "was_backed_up": false }),
        },
        // `until` is ours, and v0.1.0 ignores an unknown field: replayed
        // there it removes the one directory it always did.
        Op::Mkdir { path } => match inverse {
            Op::RemoveDir { until, .. } => json!({
                "path": path,
                "created_by_meowctl": true,
                "until": until,
            }),
            _ => json!({ "path": path, "created_by_meowctl": false }),
        },
        // Not a shape `v0.1.0` writes: it never journals these. A v0.1.0
        // binary replaying one reports "inverse not implemented" and carries
        // on, which is the same outcome it reaches today by having no record
        // at all; see [R-OPS-017].
        Op::DefaultsWrite { domain, key, .. } => match inverse {
            Op::DefaultsWrite {
                value, value_type, ..
            } => json!({
                "domain": domain,
                "key": key,
                "value": value,
                "value_type": value_type,
            }),
            _ => json!({ "domain": domain, "key": key }),
        },
        Op::PlistSet { path, key, .. } => match inverse {
            Op::PlistSet { value, .. } => json!({
                "path": path,
                "key": key,
                "value": value,
            }),
            _ => json!({ "path": path, "key": key }),
        },
        Op::Remove { .. } | Op::RemoveDir { .. } | Op::RestoreBackup { .. } | Op::Nothing => {
            json!({})
        }
    }
}

/// Replays a journal, applying inverses in reverse order.
///
/// Continues after a failure and reports which ones failed, because stopping
/// at the first leaves the rest of the run un-undone; see [R-OPS-023].
///
/// # Errors
///
/// When the journal cannot be read.
pub fn replay(
    journal: &Journal,
    fs: &dyn meowctl_fs::FileSystem,
    exec: &dyn meowctl_exec::Executor,
    events: meowctl_exec::EventSink<'_>,
) -> OpsResult<Replay> {
    let (records, broken) = journal.records()?;

    let mut applied = 0usize;
    let mut failures: Vec<(Record, OpsError)> = Vec::new();

    for error in broken {
        failures.push((
            Record {
                seq: 0,
                phase: String::new(),
                component: String::new(),
                kind: String::new(),
                inverse: serde_json::Value::Null,
            },
            error,
        ));
    }

    for record in records.into_iter().rev() {
        let outcome = inverse_from(&record).and_then(|op| op.apply(fs, exec, events));
        match outcome {
            Ok(()) => applied += 1,
            Err(e) => failures.push((record, e)),
        }
    }

    let outcome = if failures.is_empty() {
        Outcome::Ok
    } else if applied > 0 {
        Outcome::Partial
    } else {
        Outcome::Failed
    };
    Ok(Replay {
        outcome,
        applied,
        failures,
    })
}

/// Reads the journal at `path` without opening it for writing.
///
/// # Errors
///
/// When it cannot be read.
pub fn peek(path: &Path) -> OpsResult<usize> {
    Ok(Journal::open(path)?.len())
}

impl From<OpKind> for String {
    fn from(kind: OpKind) -> String {
        kind.as_str().to_owned()
    }
}
