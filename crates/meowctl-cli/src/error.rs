//! Turning a failure into an exit code.
//!
//! Here and nowhere else: a crate that knows its exit code knows about a
//! process; see [R-COMMON-031] and [R-CLI-030].

use meowctl_common::{Severity, Span};
use thiserror::Error;

/// Why a command did not do what it was asked.
#[derive(Debug, Error)]
pub enum CliError {
    /// The command line was wrong.
    #[error("{0}")]
    Usage(String),

    /// There is no configuration here.
    ///
    /// Said as itself rather than as a missing file, because the fix is a
    /// command rather than a path; see [R-CLI-050].
    #[error("no meowctl configuration in {directory}; run `meowctl init` to make one")]
    NotConfigured {
        /// Where it looked.
        directory: String,
    },

    /// Something in the configuration is wrong.
    #[error("{message}")]
    Configuration {
        /// What went wrong.
        message: String,
        /// Where, when the crate that raised it knew; see [R-CLI-031].
        span: Option<Span>,
    },

    /// A module could not be fetched or verified.
    #[error("{0}")]
    Module(String),

    /// Anything else.
    #[error("{0}")]
    General(String),
}

impl CliError {
    /// How this classifies for the exit code.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            CliError::Usage(_) => Severity::Usage,
            CliError::NotConfigured { .. } | CliError::Configuration { .. } => Severity::Config,
            CliError::Module(_) => Severity::Module,
            CliError::General(_) => Severity::General,
        }
    }

    /// Where the failure was, when that is known.
    #[must_use]
    pub const fn span(&self) -> Option<&Span> {
        match self {
            CliError::Configuration { span, .. } => span.as_ref(),
            _ => None,
        }
    }
}

impl From<meowctl_engine::EngineError> for CliError {
    fn from(error: meowctl_engine::EngineError) -> Self {
        CliError::Configuration {
            message: error.to_string(),
            span: None,
        }
    }
}

impl From<meowctl_config::ConfigError> for CliError {
    fn from(error: meowctl_config::ConfigError) -> Self {
        CliError::Configuration {
            message: error.to_string(),
            span: None,
        }
    }
}

impl From<meowctl_module::ModuleError> for CliError {
    fn from(error: meowctl_module::ModuleError) -> Self {
        CliError::Module(error.to_string())
    }
}

impl From<meowctl_common::Error> for CliError {
    fn from(error: meowctl_common::Error) -> Self {
        CliError::Configuration {
            message: error.to_string(),
            span: None,
        }
    }
}

impl From<meowctl_fs::FsError> for CliError {
    fn from(error: meowctl_fs::FsError) -> Self {
        CliError::General(error.to_string())
    }
}

/// The result of a command.
pub type CliResult<T> = Result<T, CliError>;
