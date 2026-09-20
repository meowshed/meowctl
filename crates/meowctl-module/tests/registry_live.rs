//! The published registry, fetched for real.
//!
//! Ignored by default: CI must not fail because GitHub is slow, and every
//! behaviour here is already covered against a scripted client. What this adds
//! is the one thing a scripted client cannot say, which is that the index
//! meowctl actually reads parses, and that the tarballs it names extract into
//! the shape the loader expects.
//!
//! Run it with `cargo test -p meowctl-module --test registry_live -- --ignored`.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};

use meowctl_fs::{FileSystem, MemFs};
use meowctl_module::{Cache, ModuleLoader, Roots, registry};
use meowctl_net::RealHttp;

#[test]
#[ignore = "reaches the published registry"]
fn the_published_index_parses_and_every_module_has_a_source() {
    let http = RealHttp::new();
    let index =
        registry::Index::fetch(&http, registry::DEFAULT_INDEX_URL).expect("the index fetches");
    assert!(
        !index.is_newer_than_supported(),
        "compat = {}",
        index.compat
    );
    assert!(!index.modules.is_empty(), "the index lists modules");
    for (name, entry) in &index.modules {
        assert!(!entry.source.is_empty(), "{name} has a source template");
        assert!(!entry.versions.is_empty(), "{name} has versions");
    }
}

/// The stdlib is what every configuration loads, and its release tarball is
/// built without a top-level directory. If that ever changes,
/// [R-MODULE-034] is what decides whether the module still resolves, and this
/// is where it is noticed.
#[test]
#[ignore = "reaches the published registry"]
fn the_published_stdlib_fetches_verifies_and_extracts() {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/cache")).expect("cache root");
    let http = RealHttp::new();

    let index =
        registry::Index::fetch(&http, registry::DEFAULT_INDEX_URL).expect("the index fetches");
    let entry = index.entry("stdlib").expect("the stdlib is published");
    let version = entry.latest("stdlib").expect("it has a version").to_owned();
    assert!(
        entry.integrity_for(&version).is_some(),
        "the latest stdlib carries a published hash"
    );

    let roots = Roots {
        dotfiles: PathBuf::from("/dotfiles"),
        config: PathBuf::from("/config"),
    };
    let source = ModuleLoader::new(&fs, &http, Cache::new("/cache"), roots)
        .resolved("stdlib", &version)
        .fetch("@stdlib//components/zsh")
        .expect("the component loads");

    assert!(
        std::str::from_utf8(&source).is_ok(),
        "a component is Starlark source"
    );
    assert!(
        fs.exists(
            &Path::new("/cache/stdlib")
                .join(&version)
                .join("MODULE.meow")
        )
        .expect("asking"),
        "the manifest is at the module root"
    );
}

/// [R-MODULE-034] against the real thing. GitHub serves a repository archive
/// with every entry under `repo-<ref>/`, and `v0.1.0` looks one directory
/// above where that puts the files. This is the test that says the archive
/// still has that shape.
#[test]
#[ignore = "reaches the GitHub API"]
fn a_published_github_module_resolves_its_manifest() {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/cache")).expect("cache root");
    let http = RealHttp::new();
    let roots = Roots {
        dotfiles: PathBuf::from("/dotfiles"),
        config: PathBuf::from("/config"),
    };

    let source = ModuleLoader::new(&fs, &http, Cache::new("/cache"), roots)
        .fetch("github://meowshed/meowctl-stdlib@v0.2.17//MODULE.meow")
        .expect("the manifest loads");
    let text = String::from_utf8(source).expect("the manifest is UTF-8");
    assert!(
        text.contains("module("),
        "the file is a manifest rather than whatever sits one directory up: {text}"
    );
}
