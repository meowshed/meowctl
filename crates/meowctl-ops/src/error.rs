//! What applying or undoing an operation can fail with.

use std::path::PathBuf;

use meowctl_common::Severity;

/// An operation that could not be applied, inverted, or journaled.
#[derive(Debug, thiserror::Error)]
pub enum OpsError {
    /// The filesystem refused.
    #[error(transparent)]
    Fs(#[from] meowctl_fs::FsError),

    /// A command the operation needed could not be run.
    #[error(transparent)]
    Exec(#[from] meowctl_exec::ExecError),

    /// A command the operation needed failed.
    #[error("{command} exited {code}: {stderr}")]
    CommandFailed {
        /// What was run.
        command: String,
        /// Its exit code.
        code: i32,
        /// What it said.
        stderr: String,
    },

    /// A journal record could not be written.
    ///
    /// Fails the operation rather than being logged: an effect applied with no
    /// record of how to undo it is the state this crate exists to prevent; see
    /// [R-OPS-032].
    #[error("journaling {kind}: {source}")]
    Journal {
        /// The operation that could not be recorded.
        kind: &'static str,
        /// Why not.
        source: std::io::Error,
    },

    /// A journal record could not be read back.
    ///
    /// Reported and skipped rather than fatal, because one corrupt line must
    /// not strand every earlier operation; see [R-OPS-030].
    #[error("journal record {seq} is unreadable: {reason}")]
    UnreadableRecord {
        /// Which record.
        seq: usize,
        /// What went wrong.
        reason: String,
    },

    /// A marked block was not where its record said it would be.
    #[error("the block marked {marker} is not in {path}")]
    MarkerMissing {
        /// The marker that was looked for.
        marker: String,
        /// The file it was looked for in.
        path: PathBuf,
    },
}

impl OpsError {
    /// How this classifies for the exit code.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        Severity::General
    }
}

/// The result of an operation.
pub type OpsResult<T> = Result<T, OpsError>;
