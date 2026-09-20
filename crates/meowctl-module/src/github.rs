//! Resolving a GitHub ref to a commit, and the URLs that follow from it.
//!
//! A ref is a tag or a branch, and both move. Recording the commit is what
//! makes a later fetch reproduce the same code; see [R-MODULE-011].

use meowctl_net::Http;
use serde::Deserialize;

use crate::{ModuleError, ModuleResult};

/// The GitHub API root.
pub const DEFAULT_API_BASE: &str = "https://api.github.com";

/// Where repository tarballs are served from.
pub const DEFAULT_TARBALL_BASE: &str = "https://github.com";

/// Where raw file contents are served from.
pub const DEFAULT_RAW_BASE: &str = "https://raw.githubusercontent.com";

/// The GitHub endpoints to use.
///
/// Three bases rather than one, because that is how GitHub serves them, and
/// carrying them as values is what lets a test point at a local server the way
/// `CompositeLoader`'s `githubAPIBase` does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubEndpoints {
    /// The API root, for resolving a ref.
    pub api: String,
    /// The tarball root.
    pub tarball: String,
    /// The raw-content root.
    pub raw: String,
}

impl Default for GitHubEndpoints {
    fn default() -> Self {
        GitHubEndpoints {
            api: DEFAULT_API_BASE.to_owned(),
            tarball: DEFAULT_TARBALL_BASE.to_owned(),
            raw: DEFAULT_RAW_BASE.to_owned(),
        }
    }
}

impl GitHubEndpoints {
    /// The API URL that resolves a ref to a commit.
    #[must_use]
    pub fn commit_url(&self, owner: &str, repo: &str, reference: &str) -> String {
        format!("{}/repos/{owner}/{repo}/commits/{reference}", self.api)
    }

    /// The tarball URL for a commit.
    #[must_use]
    pub fn tarball_url(&self, owner: &str, repo: &str, commit: &str) -> String {
        format!("{}/{owner}/{repo}/archive/{commit}.tar.gz", self.tarball)
    }

    /// The raw-content URL for one file at a commit.
    #[must_use]
    pub fn raw_url(&self, owner: &str, repo: &str, commit: &str, path: &str) -> String {
        format!("{}/{owner}/{repo}/{commit}/{path}", self.raw)
    }
}

/// What the commits endpoint answers with. Only the SHA is read.
#[derive(Debug, Deserialize)]
struct CommitPayload {
    sha: String,
}

/// Resolves a tag or branch to the commit it points at now.
///
/// # Errors
///
/// [`ModuleError::Fetch`] when the request fails, and
/// [`ModuleError::Archive`] when the answer carries no SHA, which means
/// something other than GitHub answered.
pub fn resolve_commit(
    http: &dyn Http,
    endpoints: &GitHubEndpoints,
    owner: &str,
    repo: &str,
    reference: &str,
) -> ModuleResult<String> {
    let module = format!("github:{owner}/{repo}@{reference}");
    let url = endpoints.commit_url(owner, repo, reference);
    let body = http.get(&url).map_err(|e| ModuleError::Fetch {
        module: module.clone(),
        what: "the commit behind the ref".to_owned(),
        source: e,
    })?;
    let payload: CommitPayload =
        serde_json::from_slice(&body).map_err(|e| ModuleError::Archive {
            module: module.clone(),
            reason: format!("the commits endpoint answered something that is not a commit: {e}"),
        })?;
    if payload.sha.is_empty() {
        return Err(ModuleError::Archive {
            module,
            reason: "the commits endpoint answered with an empty SHA".to_owned(),
        });
    }
    Ok(payload.sha)
}

#[cfg(test)]
mod tests {
    use meowctl_net::ScriptedHttp;

    use super::*;

    #[test]
    fn the_three_urls_are_the_ones_v0_1_0_builds() {
        let e = GitHubEndpoints::default();
        assert_eq!(
            e.commit_url("meowshed", "dotmeow", "v1.2.3"),
            "https://api.github.com/repos/meowshed/dotmeow/commits/v1.2.3"
        );
        assert_eq!(
            e.tarball_url("meowshed", "dotmeow", "abc123"),
            "https://github.com/meowshed/dotmeow/archive/abc123.tar.gz"
        );
        assert_eq!(
            e.raw_url("meowshed", "dotmeow", "abc123", "init.star"),
            "https://raw.githubusercontent.com/meowshed/dotmeow/abc123/init.star"
        );
    }

    #[test]
    fn a_ref_resolves_to_the_commit_it_points_at() {
        let e = GitHubEndpoints::default();
        let http = ScriptedHttp::new().with(
            e.commit_url("o", "r", "main"),
            br#"{"sha":"deadbeef","commit":{"message":"ignored"}}"#.to_vec(),
        );
        assert_eq!(
            resolve_commit(&http, &e, "o", "r", "main").expect("resolve"),
            "deadbeef"
        );
    }

    #[test]
    fn an_answer_that_is_not_a_commit_is_reported_as_such() {
        let e = GitHubEndpoints::default();
        let http = ScriptedHttp::new().with(e.commit_url("o", "r", "main"), b"not json".to_vec());
        let err = resolve_commit(&http, &e, "o", "r", "main").expect_err("should refuse");
        assert!(matches!(err, ModuleError::Archive { .. }), "{err}");
    }

    #[test]
    fn a_failed_request_names_the_module_and_what_was_wanted() {
        let e = GitHubEndpoints::default();
        let http = ScriptedHttp::new();
        let err = resolve_commit(&http, &e, "o", "r", "main").expect_err("should refuse");
        assert!(err.to_string().contains("github:o/r@main"), "{err}");
        assert!(
            err.to_string().contains("the commit behind the ref"),
            "{err}"
        );
    }
}
