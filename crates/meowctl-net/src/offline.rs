//! The implementation that makes no request.

use crate::{Http, NetError, NetResult};

/// Fails every request without making one.
///
/// This is how [R-MODULE-043] is checked. A resolution that completes against
/// `OfflineHttp` reached the network zero times, and that is a fact about the
/// implementation rather than an assertion a test can forget to write; see
/// [R-NET-014].
///
/// It is also what `--offline` will be built on, once there is a flag for it.
#[derive(Debug, Clone, Copy, Default)]
pub struct OfflineHttp;

impl Http for OfflineHttp {
    fn get(&self, url: &str) -> NetResult<Vec<u8>> {
        Err(NetError::Offline {
            url: url.to_owned(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [R-NET-014] the point of the type is that it cannot succeed.
    #[test]
    fn every_request_fails_as_offline() {
        let err = OfflineHttp.get("https://example.invalid/i").unwrap_err();
        assert!(matches!(err, NetError::Offline { .. }), "{err}");
        assert_eq!(err.url(), "https://example.invalid/i");
    }
}
