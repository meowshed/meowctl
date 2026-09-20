//! Reversible operations, and the journal that undoes them.
//!
//! An operation is data. It knows how to apply itself against a
//! [`FileSystem`] and how to produce the operation that undoes it, and the
//! journal is a log of them.
//!
//! That shape is the point. In `v0.1.0` each `ctx` method performs its effect
//! and separately calls the matching `Append*` helper on the rollback stack,
//! so a new method ships with no rollback support and nothing notices until a
//! failed run leaves a machine half configured. Here adding an effect means
//! adding a variant, and the compiler asks for its inverse.
//!
//! [`FileSystem`]: meowctl_fs::FileSystem

mod error;
mod journal;
mod op;

pub use error::{OpsError, OpsResult};
pub use journal::{Journal, Outcome, Record, Replay, inverse_from, peek, replay};
pub use op::{Op, OpKind};
