//! Fetching, verifying, extracting, and caching a module.
//!
//! Every test here runs against `MemFs` and a scripted client, so what it
//! checks is the logic rather than a network. The one thing a network would
//! add is proof that `RealHttp` works, and that is `meowctl-net`'s to make.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

mod support;

use std::path::{Path, PathBuf};

use meowctl_common::Integrity;
use meowctl_fs::{Entry, FileSystem, MemFs};
use meowctl_module::{Cache, ModuleError, ModuleLoader, ModuleUrl, Roots, registry};
use meowctl_net::{NetError, OfflineHttp, ScriptedHttp};
use support::{
    Entry as ArchiveEntry, absolute_path_tarball, escaping_tarball, github_tarball, tarball,
};

const INDEX_URL: &str = "https://registry.invalid/index.toml";
const TARBALL_URL: &str = "https://registry.invalid/stdlib/0.2.17.tar.gz";

/// A module with one component and one script in it.
fn stdlib_tarball() -> Vec<u8> {
    tarball(&[
        ArchiveEntry::file(
            "MODULE.meow",
            "module(name = \"stdlib\", version = \"0.2.17\")\n",
        ),
        ArchiveEntry::file("components/apt/init.star", "APT = 1\n"),
        ArchiveEntry::script("bin/status.sh", "#!/bin/sh\necho hi\n"),
    ])
}

/// An index listing that module, with the hash of the tarball above.
fn index_for(tarball: &[u8]) -> String {
    format!(
        r#"
compat = 1

[modules.stdlib]
versions = ["0.2.16", "0.2.17"]
source = "https://registry.invalid/{{name}}/{{version_no_v}}.tar.gz"

[modules.stdlib.integrity]
"0.2.17" = "{}"
"#,
        Integrity::compute(tarball)
    )
}

/// A filesystem with the cache root and both local roots in it.
fn tree() -> MemFs {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/cache")).expect("cache root");
    fs.create_dir_all(Path::new("/dotfiles"))
        .expect("dotfiles root");
    fs.create_dir_all(Path::new("/config"))
        .expect("config root");
    fs
}

fn roots() -> Roots {
    Roots {
        dotfiles: PathBuf::from("/dotfiles"),
        config: PathBuf::from("/config"),
    }
}

fn cache() -> Cache {
    Cache::new("/cache")
}

/// A client that serves the index and the tarball.
fn registry_http(tarball: Vec<u8>) -> ScriptedHttp {
    let index = index_for(&tarball);
    ScriptedHttp::new()
        .with(INDEX_URL, index.into_bytes())
        .with(TARBALL_URL, tarball)
}

#[test]
fn a_registry_module_is_fetched_verified_and_extracted() {
    let fs = tree();
    let http = registry_http(stdlib_tarball());
    let loader = ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17");

    let source = loader
        .fetch("@stdlib//components/apt")
        .expect("the module loads");
    assert_eq!(source, b"APT = 1\n");

    assert_eq!(http.asked(), vec![INDEX_URL, TARBALL_URL]);
    assert!(
        fs.exists(Path::new("/cache/stdlib/0.2.17/MODULE.meow"))
            .expect("the cache holds the manifest")
    );
}

/// [R-MODULE-032] a module ships scripts meowctl runs, and a dropped bit makes
/// every one of them unusable. `v0.1.0` fixed this once already.
///
/// Windows has no bit for any filesystem to carry, so what is checked there is
/// that extraction reads the entry's mode and asks for it without failing.
#[test]
fn the_executable_bit_survives_extraction() {
    let fs = tree();
    let http = registry_http(stdlib_tarball());
    ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect("the module loads");

    assert_eq!(
        fs.entry(Path::new("/cache/stdlib/0.2.17/bin/status.sh"))
            .expect("the script is there"),
        Some(Entry::File {
            len: 18,
            executable: cfg!(unix)
        })
    );
    assert_eq!(
        fs.entry(Path::new("/cache/stdlib/0.2.17/components/apt/init.star"))
            .expect("the component is there"),
        Some(Entry::File {
            len: 8,
            executable: false
        })
    );
}

/// [R-MODULE-040], [R-MODULE-041] and [R-MODULE-043]: a verified cache
/// answers without a request, and `OfflineHttp` is what proves there was
/// none.
///
/// The second loader is a fresh one over the same cache directory, so the
/// answer can only have come from a key both agree on -- the module's name
/// and its resolved version, which is what [R-MODULE-040] fixes. A key that
/// included anything per-run would miss here and fetch.
#[test]
fn a_verified_cache_needs_no_network_at_all() {
    let fs = tree();
    let http = registry_http(stdlib_tarball());
    ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect("the first load fetches");

    let offline = OfflineHttp;
    let source = ModuleLoader::new(&fs, &offline, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect("the second load is served from the cache");
    assert_eq!(source, b"APT = 1\n");
}

/// [R-MODULE-042] the cache is a copy, not the authority. Something that can
/// write to it can otherwise change what a hook runs.
#[test]
fn a_cache_whose_files_changed_is_fetched_again() {
    let fs = tree();
    let http = registry_http(stdlib_tarball());
    let load = || {
        ModuleLoader::new(&fs, &http, cache(), roots())
            .with_index_url(INDEX_URL)
            .resolved("stdlib", "0.2.17")
            .fetch("@stdlib//components/apt")
    };
    load().expect("the first load fetches");

    fs.write(
        Path::new("/cache/stdlib/0.2.17/components/apt/init.star"),
        b"APT = 666\n",
    )
    .expect("tampering with the cache");

    assert_eq!(load().expect("the tampered file is replaced"), b"APT = 1\n");
    assert_eq!(
        http.asked().len(),
        4,
        "the index and the tarball are fetched twice: {:?}",
        http.asked()
    );
}

/// [R-MODULE-042] a cache `v0.1.0` populated has the `.sri` sidecar and no
/// per-file record, and what it holds is therefore unknown.
#[test]
fn a_cache_with_no_record_is_fetched_again() {
    let fs = tree();
    fs.create_dir_all(Path::new("/cache/stdlib/0.2.17/components/apt"))
        .expect("the v0.1.0 cache layout");
    fs.write(
        Path::new("/cache/stdlib/0.2.17/components/apt/init.star"),
        b"APT = 666\n",
    )
    .expect("the v0.1.0 contents");
    fs.write(Path::new("/cache/stdlib/0.2.17/.sri"), b"sha384-whatever")
        .expect("the v0.1.0 sidecar");

    let http = registry_http(stdlib_tarball());
    let source = ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect("the module is fetched again");
    assert_eq!(source, b"APT = 1\n");
}

/// [R-MODULE-044] the sidecar `v0.1.0` reads keeps its name and its format,
/// because the two binaries share one cache directory.
#[test]
fn the_v0_1_0_sidecar_is_written_where_v0_1_0_looks_for_it() {
    let fs = tree();
    let archive = stdlib_tarball();
    let expected = Integrity::compute(&archive);
    let http = registry_http(archive);
    ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect("the module loads");

    let sidecar = fs
        .read(Path::new("/cache/stdlib/0.2.17/.sri"))
        .expect("the sidecar is written");
    assert_eq!(
        String::from_utf8(sidecar).expect("utf-8"),
        expected.to_string()
    );
}

/// [R-MODULE-013] is held by the crate rather than by a case: nothing here
/// spawns a process, because `meowctl-module` has no `Executor` to spawn one
/// with -- a machine being bootstrapped may not have `git` yet. What it has
/// is an `Http`, and [R-NET-003] is where the scheme is enforced.
///
/// [R-MODULE-030] and [R-MODULE-031]: the hash is checked before anything is
/// extracted, and the message carries both hashes so a reader can tell which
/// is which.
#[test]
fn a_tarball_that_does_not_match_the_index_is_refused_before_extraction() {
    let fs = tree();
    let index = index_for(&stdlib_tarball());
    let substituted = tarball(&[ArchiveEntry::file(
        "components/apt/init.star",
        "APT = 666\n",
    )]);
    let http = ScriptedHttp::new()
        .with(INDEX_URL, index.into_bytes())
        .with(TARBALL_URL, substituted);

    let err = ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect_err("the substituted archive is refused");

    assert!(
        matches!(err, ModuleError::IntegrityMismatch { .. }),
        "{err}"
    );
    let message = err.to_string();
    assert!(message.contains("stdlib"), "{message}");
    assert!(
        !fs.exists(Path::new("/cache/stdlib/0.2.17"))
            .expect("asking about the cache"),
        "nothing is left in the cache"
    );
}

/// [R-MODULE-033] a tarball is remote input, and an entry that climbs out of
/// the module root is how it writes somewhere it was not invited. Both forms
/// are refused: `..` and an absolute path.
#[test]
fn an_archive_entry_that_escapes_the_module_is_refused() {
    for archive in [escaping_tarball(), absolute_path_tarball()] {
        an_escaping_archive_is_refused(archive);
    }
}

fn an_escaping_archive_is_refused(archive: Vec<u8>) {
    let fs = tree();
    let index = format!(
        r#"
compat = 1
[modules.stdlib]
versions = ["0.2.17"]
source = "https://registry.invalid/{{name}}/{{version_no_v}}.tar.gz"
[modules.stdlib.integrity]
"0.2.17" = "{}"
"#,
        Integrity::compute(&archive)
    );
    let http = ScriptedHttp::new()
        .with(INDEX_URL, index.into_bytes())
        .with(TARBALL_URL, archive);

    let err = ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect_err("the escaping entry is refused");
    // Named rather than merely `Archive`, so the test cannot pass because the
    // hand-built header is corrupt rather than because the path was refused.
    assert!(
        err.to_string().contains("outside the module"),
        "the refusal is about the path: {err}"
    );
    assert!(
        !fs.exists(Path::new("/escaped")).expect("asking"),
        "nothing was written outside the cache"
    );
}

/// [R-MODULE-034] a GitHub archive puts everything under `repo-<commit>/`.
/// `v0.1.0` extracts it as it comes and then looks one directory too high, so
/// a `github:` module has never resolved its files or its manifest.
#[test]
fn a_github_archive_loses_its_single_top_level_directory() {
    let fs = tree();
    let archive = github_tarball(
        "dotmeow-abc123",
        &[
            ArchiveEntry::file("MODULE.meow", "module(name = \"dotmeow\")\n"),
            ArchiveEntry::file("init.star", "DOT = 1\n"),
        ],
    );
    let http = ScriptedHttp::new()
        .with(
            "https://api.github.com/repos/meowshed/dotmeow/commits/v1.0.0",
            br#"{"sha":"abc123"}"#.to_vec(),
        )
        .with(
            "https://github.com/meowshed/dotmeow/archive/abc123.tar.gz",
            archive,
        );

    let source = ModuleLoader::new(&fs, &http, cache(), roots())
        .fetch("github://meowshed/dotmeow@v1.0.0//init.star")
        .expect("the module loads");
    assert_eq!(source, b"DOT = 1\n");
    assert!(
        fs.exists(Path::new("/cache/meowshed/dotmeow/abc123/MODULE.meow"))
            .expect("asking"),
        "the manifest is where the loader looks for it: {:?}",
        fs.paths()
    );
}

/// [R-MODULE-011] a ref moves and a commit does not, so a recorded commit is
/// fetched without asking the API what the ref points at today.
#[test]
fn a_recorded_commit_skips_the_api_call() {
    let fs = tree();
    let archive = github_tarball(
        "dotmeow-abc123",
        &[ArchiveEntry::file("init.star", "DOT = 1\n")],
    );
    let http = ScriptedHttp::new().with(
        "https://github.com/meowshed/dotmeow/archive/abc123.tar.gz",
        archive,
    );

    ModuleLoader::new(&fs, &http, cache(), roots())
        .resolved("meowshed/dotmeow", "abc123")
        .fetch("github://meowshed/dotmeow@main//init.star")
        .expect("the module loads");
    assert_eq!(
        http.asked(),
        vec!["https://github.com/meowshed/dotmeow/archive/abc123.tar.gz"]
    );
}

/// [R-MODULE-060] the three ways an index fetch fails need three different
/// fixes, so the caller has to be able to tell them apart.
#[test]
fn an_index_that_will_not_load_says_which_failure_it_was() {
    let fs = tree();

    let unreachable = ScriptedHttp::new();
    let err = ModuleLoader::new(&fs, &unreachable, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect_err("no index");
    assert!(matches!(err, ModuleError::IndexUnavailable { .. }), "{err}");

    let refused = ScriptedHttp::new().failing(
        INDEX_URL,
        NetError::Status {
            url: INDEX_URL.to_owned(),
            code: 404,
        },
    );
    let err = ModuleLoader::new(&fs, &refused, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect_err("a 404 index");
    assert!(err.to_string().contains("404"), "{err}");

    let garbage = ScriptedHttp::new().with(INDEX_URL, b"<html>404</html>".to_vec());
    let err = ModuleLoader::new(&fs, &garbage, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect_err("an index that is not an index");
    assert!(matches!(err, ModuleError::IndexUnreadable { .. }), "{err}");
}

/// [R-MODULE-061] a module that is not published and a version that was never
/// released need different fixes, so they are different errors.
#[test]
fn a_missing_module_and_a_missing_version_are_different_failures() {
    let fs = tree();
    let http = registry_http(stdlib_tarball());

    let err = ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("ghost", "1.0.0")
        .fetch("@ghost//init.star")
        .expect_err("no such module");
    assert!(matches!(err, ModuleError::NoSuchModule { .. }), "{err}");

    let err = ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "9.9.9")
        .fetch("@stdlib//components/apt")
        .expect_err("no such version");
    assert!(matches!(err, ModuleError::NoSuchVersion { .. }), "{err}");
    assert!(err.to_string().contains("9.9.9"), "{err}");
}

/// [R-MODULE-020] a replacement is a checkout somebody is editing, so it is
/// served as it stands and never hashed.
#[test]
fn a_replaced_module_is_served_from_the_local_checkout() {
    let fs = tree();
    fs.create_dir_all(Path::new("/checkout/components/apt"))
        .expect("the checkout");
    fs.write(
        Path::new("/checkout/components/apt/init.star"),
        b"APT = local\n",
    )
    .expect("the local file");

    let offline = OfflineHttp;
    let source = ModuleLoader::new(&fs, &offline, cache(), roots())
        .with_index_url(INDEX_URL)
        .replacing("stdlib", "/checkout")
        .fetch("@stdlib//components/apt")
        .expect("the replacement is served");
    assert_eq!(source, b"APT = local\n");
}

/// [R-MODULE-064] a typo in a `replace` path must not silently fall back to
/// upstream, which is the opposite of what the directive asked for.
#[test]
fn a_replacement_that_is_not_there_fails_rather_than_fetching() {
    let fs = tree();
    let http = registry_http(stdlib_tarball());
    let err = ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .replacing("stdlib", "/nowhere")
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect_err("the missing replacement is reported");
    assert!(
        matches!(err, ModuleError::NoSuchReplacement { .. }),
        "{err}"
    );
    assert!(err.to_string().contains("/nowhere"), "{err}");
    assert!(http.asked().is_empty(), "nothing was fetched");
}

/// [R-STAR-020] a local `load()` reads from the root its scheme names, and
/// nowhere above it.
#[test]
fn a_local_load_stays_under_its_root() {
    let fs = tree();
    fs.create_dir_all(Path::new("/dotfiles/lib")).expect("lib");
    fs.write(Path::new("/dotfiles/lib/init.star"), b"HELP = 1\n")
        .expect("the helper");
    fs.write(Path::new("/secret"), b"not yours\n")
        .expect("a file above");

    let offline = OfflineHttp;
    let loader = ModuleLoader::new(&fs, &offline, cache(), roots());
    assert_eq!(
        loader.fetch("self//lib").expect("the helper loads"),
        b"HELP = 1\n"
    );

    let err = loader
        .fetch("self//../secret")
        .expect_err("the climb is refused");
    assert!(matches!(err, ModuleError::UnusableUrl { .. }), "{err}");
}

/// [R-MODULE-063] an extraction that stops partway must not leave something a
/// later run reads as a complete module. The staging directory is what makes
/// the cache directory's existence mean what every caller assumes it means.
#[test]
fn a_partial_extraction_is_not_left_where_a_module_belongs() {
    let fs = tree();
    let index = index_for(&stdlib_tarball());
    let http = ScriptedHttp::new()
        .with(INDEX_URL, index.into_bytes())
        .failing(
            TARBALL_URL,
            NetError::Body {
                url: TARBALL_URL.to_owned(),
                detail: "the connection dropped".to_owned(),
            },
        );

    let err = ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect_err("the interrupted fetch fails");
    assert!(matches!(err, ModuleError::Fetch { .. }), "{err}");
    assert!(
        !fs.exists(Path::new("/cache/stdlib/0.2.17"))
            .expect("asking"),
        "the module directory was never created: {:?}",
        fs.paths()
    );
}

/// [R-STAR-022] the check happens before the source is handed to an
/// evaluator, because a file checked afterwards has already run.
#[test]
fn a_file_changed_since_extraction_is_refused_rather_than_returned() {
    let fs = tree();
    let http = registry_http(stdlib_tarball());
    let cache = cache();
    ModuleLoader::new(&fs, &http, cache.clone(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect("the first load fetches");

    fs.write(
        Path::new("/cache/stdlib/0.2.17/components/apt/init.star"),
        b"APT = 666\n",
    )
    .expect("tampering");

    let err = cache
        .read_verified(&fs, "stdlib", "0.2.17", "components/apt/init.star")
        .expect_err("the changed file is refused");
    assert!(
        matches!(err, ModuleError::IntegrityMismatch { .. }),
        "{err}"
    );
}

/// A version the index carries no hash for is still extracted, because every
/// version published before the registry recorded hashes has none. The cache
/// record hashes what was extracted, so the module is not left unverified.
#[test]
fn a_version_with_no_published_hash_is_still_fetched_and_recorded() {
    let fs = tree();
    let archive = tarball(&[ArchiveEntry::file("init.star", "OLD = 1\n")]);
    let index = r#"
compat = 1
[modules.stdlib]
versions = ["0.2.16"]
source = "https://registry.invalid/{name}/{version_no_v}.tar.gz"
"#;
    let http = ScriptedHttp::new()
        .with(INDEX_URL, index.as_bytes().to_vec())
        .with("https://registry.invalid/stdlib/0.2.16.tar.gz", archive);

    let source = ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.16")
        .fetch("@stdlib")
        .expect("the module loads");
    assert_eq!(source, b"OLD = 1\n");
    assert!(
        cache().is_verified(&fs, "stdlib", "0.2.16"),
        "the cache records what it extracted"
    );
}

/// The default index is the one every installed meowctl reads, and a typo in
/// it would send every fetch somewhere else.
#[test]
fn the_default_index_url_is_the_published_one() {
    assert_eq!(
        registry::DEFAULT_INDEX_URL,
        "https://raw.githubusercontent.com/meowshed/meowctl-registry/main/index.toml"
    );
}

/// The parsed form of a URL is what the loader dispatches on, so a test that
/// covers the loader without covering the parse covers half of it.
#[test]
fn the_loader_refuses_a_url_it_cannot_parse() {
    let fs = tree();
    let offline = OfflineHttp;
    let err = ModuleLoader::new(&fs, &offline, cache(), roots())
        .fetch("github://nope")
        .expect_err("the malformed URL is refused");
    assert!(matches!(err, ModuleError::UnusableUrl { .. }), "{err}");
    assert!(ModuleUrl::parse("github://nope").is_err());
}

/// [R-MODULE-045] the cache is populated during a dry run.
///
/// A dry run has to evaluate a configuration's modules to produce a plan, and
/// evaluating them means fetching them. The fetch is a read as far as the
/// user's tree is concerned -- nothing in the configuration directory moves --
/// so it is not a mutation a dry run has to withhold. A dry run that refused
/// to fill the cache would report a plan it could not compute.
#[test]
fn a_fetch_fills_the_cache_whether_or_not_anything_will_be_applied() {
    let fs = tree();
    let http = registry_http(stdlib_tarball());

    ModuleLoader::new(&fs, &http, cache(), roots())
        .with_index_url(INDEX_URL)
        .resolved("stdlib", "0.2.17")
        .fetch("@stdlib//components/apt")
        .expect("the fetch");

    // The cache is a real directory in the filesystem the loader was given,
    // not something held for the duration of one loader.
    let entries = fs
        .read_dir(std::path::Path::new("/cache"))
        .expect("the cache directory exists");
    assert!(!entries.is_empty(), "nothing was cached");
}

/// [R-MODULE-033] every way an entry can climb out of the module directory.
///
/// A tarball is remote input, so this is the guard that stands between a
/// published module and the user's home directory. Table-driven because the
/// failure to catch is a deleted arm: one component kind slipping through is
/// the whole defence.
#[test]
fn every_shape_of_escaping_path_is_refused() {
    for escaping in [
        "../outside",
        "components/../../outside",
        "/absolute/path",
        "..",
        "./..",
        "a/../../b",
    ] {
        let tar = support::tarball_named(escaping);
        meowctl_module::archive::read("m", &tar)
            .expect_err(&format!("{escaping} should be refused"));
    }
}

/// [R-MODULE-033] and a path that only looks like one is kept: `./a` is `a`,
/// which is how a tar writer that prefixes everything with `./` is read.
#[test]
fn a_leading_dot_is_normalised_rather_than_refused() {
    let tar = support::tarball_named("./components/git.star");
    let files = meowctl_module::archive::read("m", &tar).expect("a normal tarball");

    assert!(
        files.iter().any(|f| f.path == "components/git.star"),
        "{:?}",
        files.iter().map(|f| &f.path).collect::<Vec<_>>()
    );
}

/// [R-MODULE-011] every part of a `github://` URL is required, and a URL
/// missing one is a mistake rather than a module with an empty name.
///
/// Table-driven for the same reason as the archive guard: each `||` in the
/// check could be flipped on its own, and one hole is the whole check.
#[test]
fn every_part_of_a_github_url_is_required() {
    for wrong in [
        "github:///repo@v1//x",
        "github://owner/@v1//x",
        "github://owner/repo@//x",
        "github://owner/repo/deeper@v1//x",
        "github://owner@v1//x",
        "github://owner/repo//x",
    ] {
        assert!(ModuleUrl::parse(wrong).is_err(), "{wrong} should not parse");
    }

    let right = ModuleUrl::parse("github://owner/repo@v1.2.3//components/git.star")
        .expect("the whole form");
    assert_eq!(
        right,
        ModuleUrl::GitHub {
            owner: "owner".to_owned(),
            repo: "repo".to_owned(),
            reference: "v1.2.3".to_owned(),
            path: "components/git.star".to_owned(),
        }
    );
}
