//! Shared vocabulary for the meowctl workspace.
//!
//! This crate sits at the bottom of the dependency graph. It depends on no
//! other workspace crate and performs no input or output, which is what lets
//! every layer above it name the same things: component identifiers, lifecycle
//! phases, module references, the error and exit-code taxonomy, and the
//! [`Event`] vocabulary the engine emits and the sinks render.
//!
//! Path resolution lives here too, rather than in `meowctl-fs`, because a dry
//! run and a real run have to resolve a path identically.

mod error;
mod event;
mod id;
pub mod paths;
mod phase;

pub use error::{Error, Severity, Span};
pub use event::{Event, Level, Outcome, PlannedStep, SkipReason, Stream};
pub use id::{ComponentId, Integrity, ModuleRef};
pub use paths::{Env, SystemEnv};
pub use phase::{Phase, PhaseSet};
