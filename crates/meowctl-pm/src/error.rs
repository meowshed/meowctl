//! Why a package declaration went nowhere.

use thiserror::Error;

/// The result of registering or dispatching.
pub type PmResult<T> = Result<T, PmError>;

/// Why a package declaration went nowhere.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PmError {
    /// Nothing handles this manager.
    ///
    /// The registered managers are listed because a typo is the common cause
    /// and the list is what makes it obvious; see [R-PM-030].
    #[error(
        "no component handles the package manager {manager}; registered: {}",
        list(registered)
    )]
    NoHandler {
        /// What the declaration named.
        manager: String,
        /// What is registered, sorted.
        registered: Vec<String>,
    },

    /// Two components claim the same manager.
    ///
    /// `v0.1.0` overwrites the earlier one, which makes the winner depend on
    /// evaluation order; see [R-PM-004].
    #[error("{first} and {second} both handle {manager}; only one component may")]
    DuplicateHandler {
        /// The manager both claim.
        manager: String,
        /// The component registered first.
        first: String,
        /// The component that tried to register second.
        second: String,
    },

    /// A handler function raised.
    ///
    /// Both components are named: the one that asked for the package and the
    /// one that handles the manager. A message naming only the handler sends
    /// the reader to a file they did not write; see [R-PM-031].
    ///
    /// Boxed because six strings would make every `Result` in this crate the
    /// size of its largest failure.
    #[error(transparent)]
    HandlerFailed(Box<HandlerFailure>),

    /// A handler function returned something of the wrong shape.
    ///
    /// A defect in the handler rather than in the configuration that used it,
    /// and coercing it would hide which; see [R-PM-032].
    #[error("{handler}'s {function} returned {found} rather than {expected}")]
    HandlerReturned {
        /// The component that handles the manager.
        handler: String,
        /// Which of its functions.
        function: String,
        /// What it returned.
        found: String,
        /// What it should have returned.
        expected: String,
    },

    /// A `repo()` reached a handler with no `add_repo`.
    #[error("{component} handles {manager} and exports no add_repo, so repo() has nowhere to go")]
    NoAddRepo {
        /// The manager.
        manager: String,
        /// The component that handles it.
        component: String,
    },
}

/// What a handler function did wrong, and who asked it to.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{component} declared {manager} {package}, and {handler}'s {function} failed: {reason}")]
pub struct HandlerFailure {
    /// The manager.
    pub manager: String,
    /// The package, or the repository, the declaration named.
    pub package: String,
    /// The component that declared it.
    pub component: String,
    /// The component that handles the manager.
    pub handler: String,
    /// Which of its functions failed.
    pub function: String,
    /// What it said.
    pub reason: String,
}

impl From<HandlerFailure> for PmError {
    fn from(failure: HandlerFailure) -> Self {
        PmError::HandlerFailed(Box::new(failure))
    }
}

/// Renders a list of manager names for a message.
fn list(names: &[String]) -> String {
    if names.is_empty() {
        return "none".to_owned();
    }
    names.join(", ")
}
