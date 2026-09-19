//! The error taxonomy and the exit codes it maps to.
//!
//! Every crate returns its own `thiserror` enum. What they share is this
//! classification, because a script branches on the exit code and the five
//! `v0.1.0` defines are the ones it already branches on; see [R-COMMON-030].

use std::fmt;

use serde::{Deserialize, Serialize};

/// What kind of failure an error is, which decides the process exit code.
///
/// Only `meowctl-cli` turns this into a code. A crate below it classifies its
/// errors and names no number; see [R-COMMON-031].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    /// Something failed that does not fit the categories below.
    General,
    /// The command line was wrong: an unknown flag, a missing argument.
    Usage,
    /// The configuration is wrong: a `.star` file that does not evaluate, a
    /// component that does not exist, a pre-rename layout.
    Config,
    /// A module could not be fetched, or its integrity did not match.
    Module,
}

impl Severity {
    /// The process exit code, from `internal/cli/errors.go`.
    #[must_use]
    pub const fn exit_code(self) -> u8 {
        match self {
            Severity::General => 1,
            Severity::Usage => 2,
            Severity::Config => 3,
            Severity::Module => 4,
        }
    }
}

/// Where in a source file an error happened.
///
/// Carried so `meowctl-cli` can render a diagnostic that points at the line.
/// `v0.1.0` reports a file name and nothing more; see [R-COMMON-032].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    /// The file the error came from.
    pub file: String,
    /// One-based line number.
    pub line: u32,
    /// One-based column number.
    pub column: u32,
    /// The source text of that line, for the caret to sit under.
    pub source_line: Option<String>,
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}

/// Failures raised by this crate.
///
/// It is deliberately small. Parsing a name from a file a user edits is the
/// only thing that happens here, so these are the only ways it can go wrong.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum Error {
    /// A component identifier that matches none of the three accepted forms.
    #[error("{value} is not a component identifier")]
    InvalidComponentId {
        /// What was parsed.
        value: String,
    },

    /// A module reference that is neither a registry name nor a GitHub source.
    #[error("{value} is not a module reference: expected a name or github:owner/repo@ref")]
    InvalidModuleRef {
        /// What was parsed.
        value: String,
    },

    /// An integrity hash that is not in the W3C Subresource Integrity form.
    #[error("{value} is not an integrity hash: expected sha384-<base64>")]
    InvalidIntegrity {
        /// What was parsed.
        value: String,
    },

    /// A phase name this build does not know.
    #[error("{name} is not a lifecycle phase")]
    UnknownPhase {
        /// The name that was read.
        name: String,
    },

    /// A path that cannot be resolved to an absolute location.
    #[error("{path} is not usable: {reason}")]
    UnusablePath {
        /// The path as given.
        path: String,
        /// Why it cannot be used.
        reason: &'static str,
    },

    /// The home directory could not be determined, so `~` cannot be expanded
    /// and the default configuration directory has no base.
    #[error("the home directory is not set: {0} is empty or missing")]
    NoHomeDirectory(&'static str),
}

impl Error {
    /// How this error classifies, which decides the exit code.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            Error::InvalidComponentId { .. }
            | Error::InvalidModuleRef { .. }
            | Error::InvalidIntegrity { .. }
            | Error::UnknownPhase { .. }
            | Error::UnusablePath { .. } => Severity::Config,
            Error::NoHomeDirectory(_) => Severity::General,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [R-COMMON-030] scripts branch on these, so the numbers are fixed by
    /// what `v0.1.0` already returns.
    #[test]
    fn the_exit_codes_match_v0_1_0() {
        assert_eq!(Severity::General.exit_code(), 1);
        assert_eq!(Severity::Usage.exit_code(), 2);
        assert_eq!(Severity::Config.exit_code(), 3);
        assert_eq!(Severity::Module.exit_code(), 4);
    }

    /// [R-COMMON-030] a malformed configuration exits 3, which is how a
    /// script tells a broken config from a broken network.
    #[test]
    fn a_parse_failure_classifies_as_a_config_error() {
        let err = Error::InvalidComponentId {
            value: "@@".to_owned(),
        };
        assert_eq!(err.severity(), Severity::Config);
        assert_eq!(err.severity().exit_code(), 3);
    }

    /// The message names what failed to parse. An error that says "invalid
    /// identifier" without the identifier sends the reader hunting.
    #[test]
    fn every_message_names_its_input() {
        assert!(
            Error::InvalidComponentId {
                value: "@@".to_owned()
            }
            .to_string()
            .contains("@@")
        );
        assert!(
            Error::InvalidModuleRef {
                value: "!!".to_owned()
            }
            .to_string()
            .contains("!!")
        );
        assert!(
            Error::UnknownPhase {
                name: "nope".to_owned()
            }
            .to_string()
            .contains("nope")
        );
    }

    #[test]
    fn a_span_renders_as_file_line_column() {
        let span = Span {
            file: "init.star".to_owned(),
            line: 12,
            column: 3,
            source_line: Some("component()".to_owned()),
        };
        assert_eq!(span.to_string(), "init.star:12:3");
    }
}
