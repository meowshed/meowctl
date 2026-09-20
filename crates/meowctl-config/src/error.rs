//! What reading or writing a configuration file can fail with.

use std::path::PathBuf;

use meowctl_common::Severity;

/// A configuration file that could not be read or written.
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    /// The file could not be read or written.
    #[error(transparent)]
    Fs(#[from] meowctl_fs::FsError),

    /// The file is not valid TOML, or does not match its schema.
    #[error("{path} is malformed: {reason}")]
    Malformed {
        /// The file.
        path: PathBuf,
        /// What the parser said, with the position it said it at.
        reason: String,
    },

    /// The file was written by a newer meowctl.
    ///
    /// Refused rather than overwritten. `v0.1.0` ignores `schema_version`
    /// entirely, so an older binary rewrites a newer file and drops whatever
    /// it did not understand, which is data loss with no message; see
    /// [R-CONFIG-041].
    #[error(
        "{path} has schema version {found}, and this meowctl understands {understood}; \
         it was written by a newer version and will not be overwritten"
    )]
    NewerSchema {
        /// The file.
        path: PathBuf,
        /// What it says.
        found: i64,
        /// What this build knows.
        understood: i64,
    },

    /// The configuration directory still has the pre-rename layout.
    #[error("{0}")]
    LegacyLayout(String),

    /// A required file is absent.
    #[error("{path} does not exist; run `meowctl init` to create one")]
    Missing {
        /// The file that was looked for.
        path: PathBuf,
    },
}

impl ConfigError {
    /// How this classifies for the exit code.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            ConfigError::Fs(_) => Severity::General,
            ConfigError::Malformed { .. }
            | ConfigError::NewerSchema { .. }
            | ConfigError::LegacyLayout(_)
            | ConfigError::Missing { .. } => Severity::Config,
        }
    }
}

/// The result of a configuration operation.
pub type ConfigResult<T> = Result<T, ConfigError>;
