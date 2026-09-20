//! Evaluating a meowctl configuration.
//!
//! The predeclared set is frozen at what `v0.1.0` accepts, down to the builtin
//! signatures: a component written for the Go binary has to run unchanged; see
//! [R-STAR-001].
//!
//! The implementation underneath is not the same. `v0.1.0` embeds
//! `go.starlark.net`; this embeds `starlark-rust`, and M0 established how the
//! two differ where it matters. The findings are in `docs/spec/starlark.md`;
//! the one that shapes this crate is that a module's heap is scoped to a
//! closure, so nothing a configuration allocates can outlive its evaluation
//! and every declaration is copied into owned data at the boundary.

// The workspace denies unsafe code. starlark-rust's derives expand to
// `unsafe impl ProvidesStaticType` and `unsafe impl Trace`, which are how the
// crate ties a Rust type to its garbage-collected heap; the safety argument
// belongs to the macro rather than to anything written here. Nothing in this
// crate writes an `unsafe` block.
#![allow(unsafe_code)]

mod accumulator;
mod builtins;
mod error;
mod evaluator;
mod json;
mod platform;

pub use accumulator::{
    Accumulator, Argument, ComponentDecl, Declarations, DepDecl, ModuleDecl, PackageAction,
    PackageDecl, ReplaceDecl, RepoDecl,
};
pub use error::{StarlarkError, StarlarkResult};
pub use evaluator::{
    Evaluated, Evaluator, HookArgument, LoadedFile, Loader, NoLoader, NoPackageManagers,
    PackageManagers,
};
pub use platform::Platform;
