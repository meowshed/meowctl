//! Which component handles which package manager, and what to call on it.
//!
//! A package manager is not code in this binary. It is a component that
//! exports a handful of functions, which is what lets Homebrew, apt and
//! everything else live in the standard library rather than in a match
//! statement here.
//!
//! Nothing in this crate calls a handler. A handler is a Starlark function in
//! a file, and calling one means evaluating that file, which is the engine's
//! job and needs the `ctx` of the component that asked for the package rather
//! than of the handler; see [R-PM-014]. What this crate produces is a [`Call`]:
//! which component, which function, and with what. `v0.1.0` keeps live
//! `Callable` values in its registry instead, which works because its heap
//! outlives every evaluation and does not translate to a heap that is scoped
//! to one.

mod error;
mod registry;

pub use error::{HandlerFailure, PmError, PmResult};
pub use registry::{Call, Handler, Registration, Registry, scan};
