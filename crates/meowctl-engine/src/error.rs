//! Why a run did not get as far as running.

use meowctl_common::{ComponentId, Severity};
use thiserror::Error;

/// The result of a stage.
pub type EngineResult<T> = Result<T, EngineError>;

/// Why a run did not get as far as running.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EngineError {
    /// The configuration's entry point could not be read or evaluated.
    #[error("{path}: {reason}")]
    Configuration {
        /// Which file.
        path: String,
        /// What went wrong.
        reason: String,
    },

    /// A component names something that is not a component identifier.
    #[error("{name} is not a component: {reason}")]
    UnusableName {
        /// What was written.
        name: String,
        /// Why it could not be read.
        reason: String,
    },

    /// The graph has a cycle.
    ///
    /// The components on it are named. `TopoSort` reports only that there is
    /// one, which leaves a user with a hundred components and no way to find
    /// the two that point at each other; see [R-ENGINE-015].
    #[error("these components depend on each other: {}", names(on_it))]
    Cycle {
        /// The components that could not be ordered, sorted.
        on_it: Vec<ComponentId>,
    },

    /// A filter named something the configuration does not declare.
    ///
    /// An empty run would look like a successful one; see [R-ENGINE-016].
    #[error("no component is named {name}")]
    NoSuchComponent {
        /// What the caller asked for.
        name: String,
    },

    /// Two components claim the same package manager, or a handler is
    /// incomplete.
    #[error(transparent)]
    PackageManager(#[from] meowctl_pm::PmError),
}

impl EngineError {
    /// How this classifies for the exit code.
    #[must_use]
    pub const fn severity(&self) -> Severity {
        // Every variant here is something wrong with the configuration: a
        // file that does not evaluate, a name that is not a component, a
        // cycle, a filter that matches nothing, two components claiming one
        // manager. None of them is a module that would not fetch.
        Severity::Config
    }
}

/// Renders a list of components for a message.
fn names(components: &[ComponentId]) -> String {
    components
        .iter()
        .map(ComponentId::as_str)
        .collect::<Vec<_>>()
        .join(", ")
}
