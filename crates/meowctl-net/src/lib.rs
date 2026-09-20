//! Every request meowctl makes, behind one trait.
//!
//! The third effect, beside [`meowctl_fs::FileSystem`] and
//! [`meowctl_exec::Executor`], and a trait for the same two reasons: a test
//! answers a request without a server, and nothing below the binary can reach
//! the network by another route.
//!
//! It knows nothing about modules, registries, or tarballs. It fetches bytes
//! and says why it could not.
//!
//! [`meowctl_fs::FileSystem`]: https://docs.rs/meowctl-fs
//! [`meowctl_exec::Executor`]: https://docs.rs/meowctl-exec

mod error;
mod offline;
mod real;
mod script;

use std::fmt::Debug;

pub use error::{NetError, NetResult};
pub use offline::OfflineHttp;
pub use real::{DEFAULT_TIMEOUT, MAX_BODY_BYTES, RealHttp};
pub use script::ScriptedHttp;

/// Fetching bytes from a URL.
///
/// One method, per [R-NET-001]. A caller that wants a document parses the
/// bytes itself, because a trait that decoded JSON would need a second method
/// for TOML and a third for a tarball, and each one would need answering in
/// every implementation.
pub trait Http: Debug {
    /// Fetches a URL and returns its body.
    ///
    /// # Errors
    ///
    /// [`NetError`], which distinguishes a refusal, an unreachable host, a
    /// status, and a body that could not be read; see [R-NET-010].
    fn get(&self, url: &str) -> NetResult<Vec<u8>>;
}

/// Sharing one client is ordinary: a resolution fetches an index, a manifest,
/// and a tarball, and they are the same connection pool.
impl<T: Http + ?Sized> Http for std::sync::Arc<T> {
    fn get(&self, url: &str) -> NetResult<Vec<u8>> {
        (**self).get(url)
    }
}
