//! The implementation that performs the request.

use std::time::Duration;

use crate::{Http, NetError, NetResult};

/// How long a request may take, start to finish.
///
/// What `v0.1.0` gives every one of its clients. A shell hook that triggers a
/// resolution is otherwise a hang with no output; see [R-NET-002].
pub const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);

/// The largest body that will be read.
///
/// `v0.1.0` calls `io.ReadAll` and has no bound, so a server that streams
/// forever is an out-of-memory kill with no message. 64 MiB is two orders of
/// magnitude above the largest module published; see [R-NET-006].
pub const MAX_BODY_BYTES: u64 = 64 * 1024 * 1024;

/// Fetches over the network.
///
/// HTTPS only, including across a redirect, which [R-NET-003] and [R-NET-005]
/// require and `v0.1.0` does not do: it builds requests with the default
/// client, so one `source` template in a registry index could downgrade every
/// module fetch to plaintext.
#[derive(Debug)]
pub struct RealHttp {
    agent: ureq::Agent,
}

impl RealHttp {
    /// A client with the default timeout.
    #[must_use]
    pub fn new() -> Self {
        Self::with_timeout(DEFAULT_TIMEOUT)
    }

    /// A client with a timeout of the caller's choosing.
    #[must_use]
    pub fn with_timeout(timeout: Duration) -> Self {
        let config = ureq::Agent::config_builder()
            .timeout_global(Some(timeout))
            // The scheme check belongs to the client rather than to each
            // caller: a redirect's target is not visible in the URL the caller
            // passed, so a caller cannot make this check for itself.
            .https_only(true)
            .build();
        RealHttp {
            agent: config.new_agent(),
        }
    }
}

impl Default for RealHttp {
    fn default() -> Self {
        Self::new()
    }
}

impl Http for RealHttp {
    fn get(&self, url: &str) -> NetResult<Vec<u8>> {
        let mut response = self.agent.get(url).call().map_err(|e| translate(url, e))?;
        let code = response.status().as_u16();
        if code != 200 {
            // `http_status_as_error` already turned 4xx and 5xx into an error.
            // This catches the rest: a 204 has no body and a 3xx here means
            // redirects were exhausted, and extracting either as a tarball
            // reports a corrupt archive rather than what happened.
            return Err(NetError::Status {
                url: url.to_owned(),
                code,
            });
        }
        response
            .body_mut()
            .with_config()
            .limit(MAX_BODY_BYTES)
            .read_to_vec()
            .map_err(|e| NetError::Body {
                url: url.to_owned(),
                detail: e.to_string(),
            })
    }
}

/// Maps a `ureq` failure onto the four cases [R-NET-010] distinguishes.
fn translate(url: &str, error: ureq::Error) -> NetError {
    let url = url.to_owned();
    match error {
        ureq::Error::StatusCode(code) => NetError::Status { url, code },
        // A URL that will not parse, and a scheme `https_only` rejected, are
        // both refusals: nothing was sent, and the fix is the URL.
        ureq::Error::BadUri(detail) => NetError::Refused { url, reason: detail },
        // A URL the `http` crate will not accept never becomes a request
        // either, and saying "could not reach" about it sends the reader
        // looking at their network.
        ureq::Error::Http(detail) => NetError::Refused {
            url,
            reason: detail.to_string(),
        },
        ureq::Error::RequireHttpsOnly(detail) => NetError::Refused { url, reason: detail },
        ureq::Error::HostNotFound => NetError::Unreachable {
            url,
            detail: "the host could not be resolved".to_owned(),
        },
        ureq::Error::Timeout(what) => NetError::Unreachable {
            url,
            detail: format!("timed out {what}"),
        },
        other => NetError::Unreachable {
            url,
            detail: other.to_string(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// [R-NET-003] the refusal happens before any request, which is why it can
    /// be checked without a server.
    #[test]
    fn a_plaintext_url_is_refused_rather_than_fetched() {
        let err = RealHttp::new().get("http://example.invalid/index.toml").unwrap_err();
        assert!(matches!(err, NetError::Refused { .. }), "{err}");
        assert_eq!(err.url(), "http://example.invalid/index.toml");
    }

    #[test]
    fn a_url_that_is_not_a_url_is_refused() {
        let err = RealHttp::new().get("not a url").unwrap_err();
        assert!(matches!(err, NetError::Refused { .. }), "{err}");
    }
}
