//! Reading a release, and refusing the ones that cannot be trusted.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use meowctl_common::Integrity;
use meowctl_release::{CHECKSUMS, Release, ReleaseError, verify};

/// What the GitHub releases API answers with, cut to what is read.
fn document(tag: &str, assets: &[&str]) -> Vec<u8> {
    let entries: Vec<String> = assets
        .iter()
        .map(|name| format!(r#"{{"name":"{name}","browser_download_url":"https://h/{name}"}}"#))
        .collect();
    format!(
        r#"{{"tag_name":"{tag}","html_url":"https://h/releases/{tag}","assets":[{}]}}"#,
        entries.join(",")
    )
    .into_bytes()
}

/// [R-CLI-072] the tag and the version are one `v` apart, which is why the
/// comparison is a function rather than an equality at the call site.
#[test]
fn a_tag_and_a_version_are_compared_across_the_v() {
    let release = Release::parse(&document("v0.2.0", &[])).expect("parse");

    assert!(release.is("0.2.0"));
    assert!(!release.is("0.1.0"));
    assert!(!release.is("v0.2.0"), "the version does not carry the v");
}

/// [R-CLI-073] the asset is the one named for this platform, and a release
/// with none says which platform was looked for and where to look.
#[test]
fn the_asset_for_a_platform_is_found_by_name() {
    let release = Release::parse(&document(
        "v0.2.0",
        &[
            "meowctl-aarch64-apple-darwin",
            "meowctl-x86_64-unknown-linux-gnu",
        ],
    ))
    .expect("parse");

    let asset = release
        .asset_for("aarch64-apple-darwin")
        .expect("the one for this machine");
    assert_eq!(asset.url, "https://h/meowctl-aarch64-apple-darwin");

    let err = release
        .asset_for("powerpc-unknown-linux-gnu")
        .expect_err("a platform nothing was built for");
    let said = err.to_string();
    assert!(said.contains("powerpc-unknown-linux-gnu"), "{said}");
    assert!(said.contains("https://h/releases/v0.2.0"), "{said}");
}

/// [R-CLI-071] a release publishing no checksums is refused, rather than
/// treated as one that needs no verification.
#[test]
fn a_release_with_no_checksums_is_refused() {
    let release =
        Release::parse(&document("v0.2.0", &["meowctl-aarch64-apple-darwin"])).expect("parse");

    let err = release.checksums().expect_err("nothing to verify against");
    assert_eq!(
        err,
        ReleaseError::NoChecksums {
            tag: "v0.2.0".to_owned()
        }
    );
}

/// [R-CLI-070] the bytes are hashed here, so the only way to pass is to be
/// the bytes the release published.
#[test]
fn bytes_that_match_what_was_published_pass() {
    let bytes = b"a binary, more or less";
    let line = format!(
        "{}  meowctl-aarch64-apple-darwin\n",
        Integrity::compute(bytes).as_str()
    );

    verify(&line, "meowctl-aarch64-apple-darwin", bytes).expect("the bytes are the bytes");
}

/// [R-CLI-070] and bytes that do not are refused, naming both hashes so a
/// reader can tell a corrupted download from a substituted one.
#[test]
fn bytes_that_do_not_match_are_refused() {
    let published = b"the real binary";
    let line = format!(
        "{}  meowctl-aarch64-apple-darwin\n",
        Integrity::compute(published).as_str()
    );

    let err = verify(&line, "meowctl-aarch64-apple-darwin", b"something else")
        .expect_err("substituted bytes");
    let said = err.to_string();
    assert!(said.contains("does not match"), "{said}");
    assert!(said.contains("sha384-"), "{said}");
}

/// [R-CLI-071] an asset with no line is unverifiable, and unverifiable is
/// refused.
#[test]
fn an_asset_with_no_line_is_refused() {
    let line = format!("{}  something-else\n", Integrity::compute(b"x").as_str());

    let err = verify(&line, "meowctl-aarch64-apple-darwin", b"x").expect_err("no line for it");
    assert_eq!(
        err,
        ReleaseError::Unlisted {
            asset: "meowctl-aarch64-apple-darwin".to_owned()
        }
    );
}

/// [R-CLI-071] a line that is not a hash and a name is a broken checksum
/// file, named by its line so it can be found.
#[test]
fn a_line_that_is_not_a_hash_and_a_name_is_refused() {
    let hash = Integrity::compute(b"x");
    let file = format!("# a comment\n\nnot-a-hash  meowctl-x\n{hash}  meowctl-y\n");

    let err = verify(&file, "meowctl-x", b"x").expect_err("a broken line");
    assert!(
        matches!(err, ReleaseError::Unparsable { line: 3, .. }),
        "{err}"
    );

    // And the rest of the file still works, because one broken line is not a
    // reason to refuse an asset it does not name.
    verify(&file, "meowctl-y", b"x").expect("the good line");
}

/// [R-CLI-071] comments and blank lines are skipped, because a checksum file
/// a human maintains will have them.
#[test]
fn comments_and_blank_lines_are_skipped() {
    let hash = Integrity::compute(b"x");
    let file = format!("# meowctl v0.2.0\n\n{hash}  meowctl-x\n\n");

    verify(&file, "meowctl-x", b"x").expect("the hash is found past the noise");
}

/// A document that is not a release says so rather than looking like an
/// empty one.
#[test]
fn a_document_that_is_not_a_release_is_refused() {
    let err = Release::parse(b"{\"nope\": true}").expect_err("not a release");
    assert!(matches!(err, ReleaseError::Unreadable { .. }), "{err}");
}

/// The checksum file is an asset like any other, so it is fetched the same
/// way.
#[test]
fn the_checksum_file_is_found_among_the_assets() {
    let release = Release::parse(&document(
        "v0.2.0",
        &["meowctl-aarch64-apple-darwin", CHECKSUMS],
    ))
    .expect("parse");

    assert_eq!(release.checksums().expect("it is there").name, CHECKSUMS);
}
