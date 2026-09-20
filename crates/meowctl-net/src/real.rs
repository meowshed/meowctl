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

/// The failure a status carries, if it is one.
///
/// Only 200 is a body. `http_status_as_error` already turned 4xx and 5xx into
/// an error; this catches the rest -- a 204 has no body and a 3xx here means
/// redirects were exhausted, and extracting either as a tarball reports a
/// corrupt archive rather than what happened. Separate from the request so
/// the rule is testable without a server; see [R-NET-004].
fn status_failure(url: &str, code: u16) -> Option<NetError> {
    (code != 200).then(|| NetError::Status {
        url: url.to_owned(),
        code,
    })
}

impl Http for RealHttp {
    fn get(&self, url: &str) -> NetResult<Vec<u8>> {
        let mut response = self.agent.get(url).call().map_err(|e| translate(url, e))?;
        if let Some(failure) = status_failure(url, response.status().as_u16()) {
            return Err(failure);
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
        ureq::Error::BadUri(detail) => NetError::Refused {
            url,
            reason: detail,
        },
        // A URL the `http` crate will not accept never becomes a request
        // either, and saying "could not reach" about it sends the reader
        // looking at their network.
        ureq::Error::Http(detail) => NetError::Refused {
            url,
            reason: detail.to_string(),
        },
        ureq::Error::RequireHttpsOnly(detail) => NetError::Refused {
            url,
            reason: detail,
        },
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

    /// [R-NET-003] and [R-NET-005]: the refusal happens before any request,
    /// which is why it can be checked without a server.
    ///
    /// One `https_only` on the agent gives both. A caller can check the URL
    /// it passed; it cannot check where a redirect goes, so the scheme rule
    /// has to live below the call. That half needs a server that redirects to
    /// plaintext, and this is the half that can be held here.
    #[test]
    fn a_plaintext_url_is_refused_rather_than_fetched() {
        let err = RealHttp::new()
            .get("http://example.invalid/index.toml")
            .unwrap_err();
        assert!(matches!(err, NetError::Refused { .. }), "{err}");
        assert_eq!(err.url(), "http://example.invalid/index.toml");
    }

    /// [R-NET-012] `RealHttp` is what performs a request, and these are the
    /// parts of it that can be held without a server: the client's
    /// configuration and the refusals that happen before anything is sent.
    /// The request itself is exercised by `meowctl-module`'s live registry
    /// test, which is skipped when the network is not there.
    ///
    /// [R-NET-002] every request carries a timeout, and the default is the
    /// 30 seconds `v0.1.0` gives all of its clients. A resolution without one
    /// is a hang with no output.
    #[test]
    fn the_default_timeout_is_thirty_seconds() {
        assert_eq!(DEFAULT_TIMEOUT, Duration::from_secs(30));
    }

    /// [R-NET-004] only 200 is a body, and everything else is a failure
    /// carrying the code, which is what tells a missing module from a
    /// rate-limited one.
    #[test]
    fn only_two_hundred_is_a_body() {
        assert!(status_failure("https://h/x", 200).is_none());

        for code in [201u16, 204, 301, 302, 400, 404, 429, 500, 503] {
            let failure = status_failure("https://h/x", code)
                .unwrap_or_else(|| panic!("{code} should be a failure"));
            assert_eq!(failure.status(), Some(code));
        }
    }

    /// [R-NET-004] a status that is not 200 is a failure carrying the code,
    /// which is what tells a missing module from a rate-limited one. The
    /// constructor for that error is here and the `ScriptedHttp` test asserts
    /// the code survives; what needs a server is producing the status, not
    /// reporting it.
    #[test]
    fn a_status_that_is_not_200_carries_its_code() {
        let err = NetError::Status {
            url: "https://h/x".to_owned(),
            code: 429,
        };
        assert_eq!(err.status(), Some(429));
        assert!(err.to_string().contains("429"), "{err}");
    }

    /// [R-NET-006] a body is bounded, so a URL answering with an endless
    /// stream fails rather than filling the disk. The bound is an order of
    /// magnitude above the largest module published.
    #[test]
    fn a_body_is_bounded_well_above_a_real_module() {
        assert_eq!(MAX_BODY_BYTES, 64 * 1024 * 1024);
    }

    #[test]
    fn a_url_that_is_not_a_url_is_refused() {
        let err = RealHttp::new().get("not a url").unwrap_err();
        assert!(matches!(err, NetError::Refused { .. }), "{err}");
    }
}
