//! Evaluation failures, with the position to point at.

use meowctl_common::{Severity, Span};

/// A configuration that did not evaluate.
#[derive(Debug, thiserror::Error)]
pub enum StarlarkError {
    /// The file does not parse, or evaluating it failed.
    ///
    /// M0 found that `starlark-rust` already carries a span and renders a
    /// caret under the offending source, which `v0.1.0` does not: its errors
    /// name a file and nothing more; see [R-STAR-040].
    #[error("{message}")]
    Evaluation {
        /// What went wrong, as the evaluator put it.
        message: String,
        /// Where, when the evaluator knew.
        span: Option<Span>,
        /// The files that were loading when it happened, outermost first.
        ///
        /// A failure in a shared standard-library helper is useless without
        /// the component that loaded it; see [R-STAR-041].
        load_chain: Vec<String>,
    },

    /// A `load()` named a module the loader could not produce.
    ///
    /// Classified as a module failure rather than a configuration one, because
    /// a script branching on the exit code needs to tell a broken network from
    /// a broken configuration; see [R-STAR-053].
    #[error("loading {module}: {reason}")]
    Load {
        /// What was asked for.
        module: String,
        /// Why it could not be produced.
        reason: String,
    },

    /// A global that should have been callable is not.
    #[error("{component} exports {hook}, but it is a {found} rather than a function")]
    NotCallable {
        /// The component.
        component: String,
        /// The hook name.
        hook: String,
        /// What the global actually is.
        found: String,
    },
}

impl StarlarkError {
    /// Where the failure was, when that is known.
    #[must_use]
    pub const fn span(&self) -> Option<&Span> {
        match self {
            StarlarkError::Evaluation { span, .. } => span.as_ref(),
            StarlarkError::Load { .. } | StarlarkError::NotCallable { .. } => None,
        }
    }

    /// How this classifies for the exit code.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        match self {
            // A module that will not fetch is not a configuration mistake.
            StarlarkError::Load { .. } => Severity::Module,
            StarlarkError::Evaluation { .. } | StarlarkError::NotCallable { .. } => {
                Severity::Config
            }
        }
    }
}

/// The result of evaluating.
pub type StarlarkResult<T> = Result<T, StarlarkError>;
