//! What running a command can fail with.
//!
//! A non-zero exit is not here: that is an [`Output`], which the caller reads.
//! An error means the command could not be run at all.
//!
//! [`Output`]: crate::Output

use meowctl_common::Severity;

/// A command that could not be run.
#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    /// The program is not on `PATH`.
    ///
    /// Distinct from a general spawn failure because it is the common case: a
    /// hook calling a package manager that is not installed. "no such file or
    /// directory" without the program name sends the reader hunting; see
    /// [R-EXEC-030].
    #[error("{program} is not on PATH")]
    NotOnPath {
        /// The program that was looked for.
        program: String,
    },

    /// The command could not be started, or its output could not be read.
    #[error("running {program}: {source}")]
    Spawn {
        /// The program that was to be run.
        program: String,
        /// The underlying failure.
        source: std::io::Error,
    },

    /// A scripted executor was asked for a command nobody scripted.
    ///
    /// A test failure rather than a runtime one: it means the code under test
    /// ran something the test did not expect, which is worth knowing.
    #[error("no scripted result for `{command}`")]
    Unscripted {
        /// The command line as it would have been run.
        command: String,
    },
}

impl ExecError {
    /// How this classifies for the exit code.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        Severity::General
    }
}

/// The result of running a command.
pub type ExecResult<T> = Result<T, ExecError>;
