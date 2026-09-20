//! Why a hook's call to `ctx` failed.

use thiserror::Error;

/// The result of a `ctx` method.
pub type CtxResult<T> = Result<T, CtxError>;

/// Why a `ctx` method failed.
///
/// Every variant names the method, because a hook that calls eight of them and
/// fails has to say which; see [R-CTX-040].
#[derive(Debug, Error)]
pub enum CtxError {
    /// An argument was the wrong shape.
    #[error("ctx.{method}: {argument} must be {expected}")]
    Argument {
        /// The method.
        method: &'static str,
        /// The argument.
        argument: String,
        /// What it should have been.
        expected: &'static str,
    },

    /// A path could not be used.
    #[error("ctx.{method}: {source}")]
    Path {
        /// The method.
        method: &'static str,
        /// What was wrong with it.
        #[source]
        source: meowctl_common::Error,
    },

    /// An effect failed.
    #[error("ctx.{method}: {reason}")]
    Effect {
        /// The method.
        method: &'static str,
        /// What went wrong.
        reason: String,
    },
}
