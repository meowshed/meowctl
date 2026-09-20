//! The implementation a test supplies.

use std::collections::HashMap;
use std::sync::Mutex;

use crate::{Http, NetError, NetResult};

/// Answers from a table, and records what it was asked.
///
/// A URL the table does not hold fails rather than returning empty bytes, per
/// [R-NET-013]: a test that silently gets nothing back passes for the wrong
/// reason, and an empty tarball extracts to an empty module.
#[derive(Debug, Default)]
pub struct ScriptedHttp {
    responses: Mutex<HashMap<String, NetResult<Vec<u8>>>>,
    asked: Mutex<Vec<String>>,
}

impl ScriptedHttp {
    /// An empty table. Every request fails until one is added.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Answers `url` with `body`.
    #[must_use]
    pub fn with(self, url: impl Into<String>, body: impl Into<Vec<u8>>) -> Self {
        self.responses
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(url.into(), Ok(body.into()));
        self
    }

    /// Fails `url` with `error`.
    #[must_use]
    pub fn failing(self, url: impl Into<String>, error: NetError) -> Self {
        self.responses
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .insert(url.into(), Err(error));
        self
    }

    /// Every URL asked for, in order, including the ones that failed.
    #[must_use]
    pub fn asked(&self) -> Vec<String> {
        self.asked
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }
}

impl Http for ScriptedHttp {
    fn get(&self, url: &str) -> NetResult<Vec<u8>> {
        self.asked
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .push(url.to_owned());
        self.responses
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .get(url)
            .cloned()
            .unwrap_or_else(|| {
                Err(NetError::Unreachable {
                    url: url.to_owned(),
                    detail: "no response was scripted for this URL".to_owned(),
                })
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_scripted_url_answers_and_is_recorded() {
        let http = ScriptedHttp::new().with("https://h/a", b"one".to_vec());
        assert_eq!(http.get("https://h/a").unwrap(), b"one");
        assert_eq!(http.get("https://h/a").unwrap(), b"one");
        assert_eq!(http.asked(), vec!["https://h/a", "https://h/a"]);
    }

    /// [R-NET-013] an unscripted URL is a test that did not mean what it said.
    #[test]
    fn an_unscripted_url_fails_rather_than_returning_nothing() {
        let http = ScriptedHttp::new();
        let err = http.get("https://h/missing").unwrap_err();
        assert!(
            err.to_string().contains("no response was scripted"),
            "{err}"
        );
        assert_eq!(http.asked(), vec!["https://h/missing"]);
    }

    #[test]
    fn a_scripted_failure_is_returned() {
        let http = ScriptedHttp::new().failing(
            "https://h/gone",
            NetError::Status {
                url: "https://h/gone".to_owned(),
                code: 404,
            },
        );
        assert_eq!(http.get("https://h/gone").unwrap_err().status(), Some(404));
    }
}
