//! The registry index.
//!
//! One TOML document listing every published module: its versions, a URL
//! template for the tarball, and a hash per version. `index.toml` in
//! `meowshed/meowctl-registry` is the one meowctl reads by default.

use std::collections::BTreeMap;

use meowctl_common::Integrity;
use meowctl_net::Http;
use serde::Deserialize;

use crate::{ModuleError, ModuleResult};

/// Where the index lives when nobody says otherwise.
///
/// The same URL `defaultRegistryURL` names in
/// `internal/starlark/loader/registry.go`.
pub const DEFAULT_INDEX_URL: &str =
    "https://raw.githubusercontent.com/meowshed/meowctl-registry/main/index.toml";

/// The schema version this binary understands.
///
/// A higher one in the index is a warning rather than a failure, which is what
/// `v0.1.0` does: an index that grew a field this binary ignores is still an
/// index it can read.
pub const SUPPORTED_COMPAT: i64 = 1;

/// One module's entry in the index.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct IndexEntry {
    /// Every published version, ascending. The last is the latest.
    #[serde(default)]
    pub versions: Vec<String>,
    /// A URL template for the tarball.
    ///
    /// `{name}`, `{version}`, and `{version_no_v}` are substituted; see
    /// [`source_url`].
    #[serde(default)]
    pub source: String,
    /// The tarball hash for each version, where the publisher recorded one.
    ///
    /// Absent for versions published before the registry carried hashes,
    /// which `v0.1.0` skips rather than refusing; see [`IndexEntry::integrity_for`].
    #[serde(default)]
    pub integrity: BTreeMap<String, String>,
}

impl IndexEntry {
    /// The recorded hash for a version, when there is one and it parses.
    ///
    /// A hash the index carries in some form that is not SRI is treated as
    /// absent rather than as a mismatch, because refusing would make a
    /// malformed entry in someone else's index unfixable from here. The cache
    /// record still hashes what was extracted, so the module is not
    /// unverified afterwards; see [R-MODULE-044].
    #[must_use]
    pub fn integrity_for(&self, version: &str) -> Option<Integrity> {
        self.integrity.get(version)?.parse().ok()
    }

    /// The latest published version.
    ///
    /// # Errors
    ///
    /// [`ModuleError::NoVersions`] when the entry lists none, which `v0.1.0`
    /// also refuses rather than treating as an empty module.
    pub fn latest(&self, module: &str) -> ModuleResult<&str> {
        self.versions
            .last()
            .map(String::as_str)
            .ok_or_else(|| ModuleError::NoVersions {
                module: module.to_owned(),
            })
    }
}

/// The index document.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Index {
    /// The schema version. A missing field decodes as 0, which `v0.1.0`
    /// treats the same as 1.
    #[serde(default)]
    pub compat: i64,
    /// Every module, by name.
    #[serde(default)]
    pub modules: BTreeMap<String, IndexEntry>,
}

impl Index {
    /// Parses an index document.
    ///
    /// # Errors
    ///
    /// [`ModuleError::IndexUnreadable`] when the bytes are not an index.
    pub fn parse(url: &str, body: &[u8]) -> ModuleResult<Index> {
        let text = std::str::from_utf8(body).map_err(|e| ModuleError::IndexUnreadable {
            url: url.to_owned(),
            reason: format!("it is not UTF-8: {e}"),
        })?;
        toml::from_str(text).map_err(|e| ModuleError::IndexUnreadable {
            url: url.to_owned(),
            reason: e.message().to_owned(),
        })
    }

    /// Fetches and parses the index.
    ///
    /// The three failures [R-MODULE-060] distinguishes are three variants
    /// here: the request, the status inside it, and the parse.
    ///
    /// # Errors
    ///
    /// [`ModuleError::IndexUnavailable`] or [`ModuleError::IndexUnreadable`].
    pub fn fetch(http: &dyn Http, url: &str) -> ModuleResult<Index> {
        let body = http.get(url).map_err(|e| ModuleError::IndexUnavailable {
            url: url.to_owned(),
            source: e,
        })?;
        Index::parse(url, &body)
    }

    /// One module's entry.
    ///
    /// # Errors
    ///
    /// [`ModuleError::NoSuchModule`], which [R-MODULE-061] wants distinct from
    /// a version that does not exist.
    pub fn entry(&self, module: &str) -> ModuleResult<&IndexEntry> {
        self.modules
            .get(module)
            .ok_or_else(|| ModuleError::NoSuchModule {
                module: module.to_owned(),
            })
    }

    /// Whether the index declares a schema this binary does not know.
    ///
    /// Reported rather than refused, and the caller decides how to say so.
    #[must_use]
    pub const fn is_newer_than_supported(&self) -> bool {
        self.compat > SUPPORTED_COMPAT
    }
}

/// Builds a tarball URL from an index template.
///
/// `{name}` is the module, `{version}` always carries a leading `v`, and
/// `{version_no_v}` never does. `buildSourceURL` is where both conventions
/// come from, and templates in the published index use each of them.
#[must_use]
pub fn source_url(template: &str, name: &str, version: &str) -> String {
    let with_v = if version.starts_with('v') {
        version.to_owned()
    } else {
        format!("v{version}")
    };
    template
        .replace("{name}", name)
        .replace("{version}", &with_v)
        .replace("{version_no_v}", version.trim_start_matches('v'))
}

#[cfg(test)]
mod tests {
    use super::*;

    const INDEX: &str = r#"
compat = 1

[modules.stdlib]
repo = "https://github.com/meowshed/meowctl-stdlib"
versions = ["0.2.15", "0.2.16", "0.2.17"]
source = "https://github.com/meowshed/meowctl-{name}/archive/refs/tags/{version}.tar.gz"

[modules.stdlib.integrity]
"0.2.17" = "sha384-AAA"
"#;

    #[test]
    fn an_index_parses_with_the_fields_v0_1_0_reads() {
        let index = Index::parse("u", INDEX.as_bytes()).expect("parse");
        assert!(!index.is_newer_than_supported());
        let entry = index.entry("stdlib").expect("entry");
        assert_eq!(entry.latest("stdlib").expect("latest"), "0.2.17");
        assert_eq!(
            entry.integrity_for("0.2.17").map(|i| i.to_string()),
            Some("sha384-AAA".to_owned())
        );
        assert!(entry.integrity_for("0.2.16").is_none());
    }

    /// A field the index grows that this binary does not read must not stop
    /// it reading the rest: `repo` above is exactly that, and every published
    /// entry carries one.
    #[test]
    fn an_unknown_field_is_ignored() {
        assert!(Index::parse("u", INDEX.as_bytes()).is_ok());
    }

    /// [R-MODULE-060] a body that is not an index is the third of the three
    /// failures, and it has to be distinguishable from the other two.
    #[test]
    fn a_body_that_is_not_an_index_says_so() {
        let err = Index::parse("u", b"<html>404</html>").expect_err("should refuse");
        assert!(matches!(err, ModuleError::IndexUnreadable { .. }), "{err}");
    }

    #[test]
    fn a_newer_schema_is_reported_rather_than_refused() {
        let index = Index::parse("u", b"compat = 2\n").expect("parse");
        assert!(index.is_newer_than_supported());
    }

    /// Both spellings appear in the published index, and getting one wrong
    /// fetches a 404 from a URL that looks almost right.
    #[test]
    fn a_source_template_takes_both_version_spellings() {
        assert_eq!(
            source_url("https://h/{name}/{version}.tar.gz", "stdlib", "0.2.17"),
            "https://h/stdlib/v0.2.17.tar.gz"
        );
        assert_eq!(
            source_url(
                "https://h/{name}/{version_no_v}.tar.gz",
                "stdlib",
                "v0.2.17"
            ),
            "https://h/stdlib/0.2.17.tar.gz"
        );
    }

    #[test]
    fn a_module_that_is_not_listed_is_named() {
        let index = Index::parse("u", INDEX.as_bytes()).expect("parse");
        let err = index.entry("ghost").expect_err("should refuse");
        assert!(err.to_string().contains("ghost"), "{err}");
    }
}
