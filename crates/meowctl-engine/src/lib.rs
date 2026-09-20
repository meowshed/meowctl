//! What happens, and making it happen.
//!
//! The engine discovers components, builds their graph, computes a plan, runs
//! the phases, tracks what completed, and drives rollback when something
//! fails. It emits [`Event`]s and holds no renderer: `buildRunner` in
//! `v0.1.0` takes a `tui.Writer`, and the cost of that is `SuspendOutput`, a
//! renderer concern threaded into the Starlark layer as a callback.
//!
//! The stages are distinct types, so one cannot be entered before the stage
//! it depends on has produced its value; see [R-ENGINE-001]. `v0.1.0`
//! expresses the same ordering in comments, which is how "pass one registers
//! the package-manager handlers" became something a caller can forget.
//!
//! [`Event`]: meowctl_common::Event

mod discovery;
mod error;
mod graph;
mod plan;
mod progress;
mod runner;

pub use discovery::{Component, ComponentSource, Declaration, Discovered, Sources, discover};
pub use error::{EngineError, EngineResult};
pub use graph::Graph;
pub use plan::{Inputs, Plan};
pub use progress::{Progress, fingerprints, interrupted_run, stale_components};
pub use runner::{Failure, Report, Runner, Settings};
