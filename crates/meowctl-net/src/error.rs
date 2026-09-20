//! Why a request did not produce bytes.

use thiserror::Error;

/// The result of a request.
pub type NetResult<T> = Result<T, NetError>;

/// Why a request did not produce bytes.
///
/// Five variants for five different fixes. [R-MODULE-060] requires a module
/// resolution to say whether an index failed on the network, on the status, or
/// on the parse, and it can only say what it is told; see [R-NET-010].
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum NetError {
    /// The URL was rejected before anything was sent.
    ///
    /// A scheme that is not `https`, or a URL that does not parse; see
    /// [R-NET-003].
    #[error("refused to fetch {url}: {reason}")]
    Refused {
        /// The URL, as the caller wrote it.
        url: String,
        /// Why it was refused.
        reason: String,
    },

    /// Nothing answered.
    #[error("could not reach {url}: {detail}")]
    Unreachable {
        /// The URL.
        url: String,
        /// What the transport said.
        detail: String,
    },

    /// Something answered, with a status that is not 200.
    #[error("{url} answered HTTP {code}")]
    Status {
        /// The URL.
        url: String,
        /// The status code.
        code: u16,
    },

    /// The response started and did not finish, or ran past the size bound.
    #[error("could not read the body of {url}: {detail}")]
    Body {
        /// The URL.
        url: String,
        /// What went wrong reading it.
        detail: String,
    },

    /// No request was attempted, because this implementation makes none.
    ///
    /// What [`OfflineHttp`] returns; see [R-NET-014].
    ///
    /// [`OfflineHttp`]: crate::OfflineHttp
    #[error("{url} was not fetched: this run makes no network requests")]
    Offline {
        /// The URL that would have been fetched.
        url: String,
    },
}

impl NetError {
    /// The URL the failure is about.
    ///
    /// Every variant carries one, per [R-NET-011], and a caller reporting a
    /// resolution failure wants it without matching on five variants.
    #[must_use]
    pub fn url(&self) -> &str {
        match self {
            NetError::Refused { url, .. }
            | NetError::Unreachable { url, .. }
            | NetError::Status { url, .. }
            | NetError::Body { url, .. }
            | NetError::Offline { url } => url,
        }
    }

    /// The status code, when the server answered with one.
    #[must_use]
    pub const fn status(&self) -> Option<u16> {
        match self {
            NetError::Status { code, .. } => Some(*code),
            _ => None,
        }
    }
}
