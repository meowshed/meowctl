//! The `ctx` value a lifecycle hook is called with.
//!
//! The surface a component author writes against, and a binding and nothing
//! more: each method validates its arguments, builds an [`Op`] or calls the
//! [`Executor`], and returns. `internal/ctx/methods.go` is 1206 lines mixing
//! Starlark glue, filesystem mutation, process execution and rollback
//! journaling, which is why a method could ship there without an inverse and
//! why every one of them repeated a dry-run check.
//!
//! Nothing here asks whether this is a dry run. The [`FileSystem`] and
//! [`Executor`] it was given decide that; see [R-CTX-014].
//!
//! [`Op`]: meowctl_ops::Op
//! [`Executor`]: meowctl_exec::Executor
//! [`FileSystem`]: meowctl_fs::FileSystem

// The `ProvidesStaticType` derive expands to an `unsafe impl`, which the
// workspace denies. The alternative is not writing Starlark values.
#![allow(unsafe_code)]

mod error;
mod state;
mod template;
mod value;

pub use error::{CtxError, CtxResult};
pub use state::{Capabilities, Core, Effects, Surface};
pub use value::{Ctx, Restricted};
