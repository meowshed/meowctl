//! Resolving a manifest into a lock.
//!
//! The parity test is the one that matters: the fixture was produced by
//! `v0.1.0`'s own `SyncModules` against a scripted registry, and the bytes
//! here have to match it outside the `meta` table; see [R-CONFIG-023].

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

mod support;

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use meowctl_config::{Dep, LockFile, ModuleEntry, Replace};
use meowctl_fs::{FileSystem, MemFs};
use meowctl_module::{Cache, ModuleError, Syncer, Upgrade, overlay_replaces};
use meowctl_net::{OfflineHttp, ScriptedHttp};
use support::{Entry as ArchiveEntry, github_tarball, tarball};

/// The lock `v0.1.0` wrote from the manifest below, against the registry
/// below. Regenerate with
/// `MEOWCTL_WRITE_FIXTURE=1 go test ./internal/starlark/loader -run TestWriteSyncFixture`.
const V0_1_0_SYNCED: &str = include_str!("../../../tests/compat/fixtures/deps.lock.synced");

const REGISTRY: &str = "https://registry.invalid";
const INDEX_URL: &str = "https://registry.invalid/index.toml";
const CHECKOUT: &str = "/checkout";

/// The tarball `v0.1.0` hashed when it wrote the fixture.
///
/// The bytes are checked in beside the lock rather than rebuilt here, because
/// two `tar` and `gzip` implementations do not produce the same file from the
/// same tree, and then the two locks would agree on every field but the
/// hashes -- which are the fields a lock exists to carry.
fn stdlib() -> Vec<u8> {
    include_bytes!("../../../tests/compat/fixtures/synced-stdlib-0.2.17.tar.gz").to_vec()
}

fn helper() -> Vec<u8> {
    include_bytes!("../../../tests/compat/fixtures/synced-helper-1.1.0.tar.gz").to_vec()
}

/// The same index the Go fixture generator served.
fn index(stdlib: &[u8], helper: &[u8]) -> String {
    format!(
        r#"compat = 1

[modules.stdlib]
versions = ["0.2.16", "0.2.17"]
source = "{REGISTRY}/t/{{name}}-{{version_no_v}}.tar.gz"

[modules.stdlib.integrity]
"0.2.17" = "{}"

[modules.helper]
versions = ["1.0.0", "1.1.0"]
source = "{REGISTRY}/t/{{name}}-{{version_no_v}}.tar.gz"

[modules.helper.integrity]
"1.1.0" = "{}"
"#,
        meowctl_common::Integrity::compute(stdlib),
        meowctl_common::Integrity::compute(helper)
    )
}

fn registry_http() -> ScriptedHttp {
    let stdlib = stdlib();
    let helper = helper();
    let index = index(&stdlib, &helper);
    ScriptedHttp::new()
        .with(INDEX_URL, index.into_bytes())
        .with(format!("{REGISTRY}/t/stdlib-0.2.17.tar.gz"), stdlib)
        .with(format!("{REGISTRY}/t/helper-1.1.0.tar.gz"), helper)
}

fn tree() -> MemFs {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/cache")).expect("cache root");
    fs.create_dir_all(Path::new(CHECKOUT))
        .expect("the checkout");
    fs
}

fn dep(name: &str, version: &str) -> Dep {
    Dep {
        name: name.to_owned(),
        version: version.to_owned(),
        source: String::new(),
    }
}

fn from_source(name: &str, source: &str) -> Dep {
    Dep {
        name: name.to_owned(),
        version: String::new(),
        source: source.to_owned(),
    }
}

/// The lock rendered, with the `meta` table dropped.
///
/// [R-CONFIG-023] excludes it: it is the one table the two binaries are meant
/// to disagree about, and [R-CONFIG-022] says why.
fn without_meta(fs: &MemFs, lock: &LockFile) -> String {
    lock.write(fs, Path::new("/cfg/deps.lock"))
        .expect("the lock writes");
    let text = String::from_utf8(
        fs.read(Path::new("/cfg/deps.lock"))
            .expect("the lock reads back"),
    )
    .expect("the lock is UTF-8");
    let modules = text
        .find("[modules]")
        .expect("the lock has a modules table");
    text[modules..].to_owned()
}

/// [R-CONFIG-023] and [R-MODULE-051]: the same manifest against the same
/// registry produces the lock `v0.1.0` produced, byte for byte.
#[test]
fn the_lock_matches_the_one_v0_1_0_writes() {
    let fs = tree();
    fs.create_dir_all(Path::new("/cfg")).expect("config dir");
    let http = registry_http();

    let synced = Syncer::new(
        &fs,
        &http,
        Cache::new("/cache"),
        "0.2.0",
        "2026-09-20T00:00:00Z",
    )
    .with_index_url(INDEX_URL)
    .sync(
        &[
            dep("stdlib", "0.2.17"),
            dep("helper", "1.1.0"),
            dep("forked", "0.1.0"),
        ],
        &[Replace {
            name: "forked".to_owned(),
            path: CHECKOUT.to_owned(),
            source: String::new(),
        }],
        &LockFile::default(),
        &Upgrade::Nothing,
    )
    .expect("the manifest syncs");

    let expected = V0_1_0_SYNCED
        .replace("{registry}", REGISTRY)
        .replace("{checkout}", CHECKOUT);
    let expected = &expected[expected.find("[modules]").expect("the fixture has modules")..];

    assert_eq!(without_meta(&fs, &synced.lock), expected);
}

/// [R-CONFIG-022] the two fields `v0.1.0` declares and never fills are what
/// tell a user which binary last wrote their lock.
#[test]
fn the_lock_records_which_binary_wrote_it() {
    let fs = tree();
    fs.create_dir_all(Path::new("/cfg")).expect("config dir");
    let http = registry_http();
    let synced = Syncer::new(
        &fs,
        &http,
        Cache::new("/cache"),
        "0.2.0",
        "2026-09-20T00:00:00Z",
    )
    .with_index_url(INDEX_URL)
    .sync(
        &[dep("stdlib", "0.2.17")],
        &[],
        &LockFile::default(),
        &Upgrade::Nothing,
    )
    .expect("the manifest syncs");
    assert_eq!(synced.lock.meta.generated_by, "0.2.0");
    assert_eq!(synced.lock.meta.updated_at, "2026-09-20T00:00:00Z");
}

/// [R-MODULE-050] resolution that ran again on every sync would move with
/// whatever the registry published today, which is what a lock exists to stop.
#[test]
fn a_locked_version_is_kept_even_when_a_newer_one_is_published() {
    let fs = tree();
    let http = registry_http();
    let previous = LockFile {
        modules: BTreeMap::from([(
            "stdlib".to_owned(),
            ModuleEntry {
                version: "0.2.16".to_owned(),
                ..ModuleEntry::default()
            },
        )]),
        ..LockFile::default()
    };

    // 0.2.16 has no published hash and is not in the scripted responses, so a
    // sync that honoured the lock has to fetch it; that it asks for it at all
    // is the point.
    let err = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "latest")],
            &[],
            &previous,
            &Upgrade::Nothing,
        )
        .expect_err("0.2.16 is not served");
    assert!(err.to_string().contains("0.2.16"), "{err}");
    assert!(
        http.asked()
            .iter()
            .any(|url| url.ends_with("stdlib-0.2.16.tar.gz")),
        "the locked version was the one asked for: {:?}",
        http.asked()
    );
}

/// [R-MODULE-053] an upgrade of one module must not move the others, because
/// `meowctl dep upgrade stdlib` says nothing about anything else.
#[test]
fn an_upgrade_moves_only_the_modules_it_names() {
    let fs = tree();
    let http = registry_http();
    let previous = LockFile {
        modules: BTreeMap::from([
            (
                "stdlib".to_owned(),
                ModuleEntry {
                    version: "0.2.16".to_owned(),
                    ..ModuleEntry::default()
                },
            ),
            (
                "helper".to_owned(),
                ModuleEntry {
                    version: "1.1.0".to_owned(),
                    ..ModuleEntry::default()
                },
            ),
        ]),
        ..LockFile::default()
    };

    let synced = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "latest"), dep("helper", "latest")],
            &[],
            &previous,
            &Upgrade::Named(BTreeSet::from(["stdlib".to_owned()])),
        )
        .expect("the upgrade resolves");

    assert_eq!(
        synced.resolved.get("stdlib").map(String::as_str),
        Some("0.2.17")
    );
    assert_eq!(
        synced.resolved.get("helper").map(String::as_str),
        Some("1.1.0")
    );
}

/// [R-MODULE-050] and [R-MODULE-053]: ignoring the lock entirely is what
/// `--update` means, and it moves everything to what the index publishes now.
#[test]
fn ignoring_the_lock_resolves_everything_again() {
    let fs = tree();
    let http = registry_http();
    let previous = LockFile {
        modules: BTreeMap::from([(
            "stdlib".to_owned(),
            ModuleEntry {
                version: "0.2.16".to_owned(),
                ..ModuleEntry::default()
            },
        )]),
        ..LockFile::default()
    };

    let synced = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "latest")],
            &[],
            &previous,
            &Upgrade::Everything,
        )
        .expect("the resolution runs again");
    assert_eq!(
        synced.resolved.get("stdlib").map(String::as_str),
        Some("0.2.17")
    );
}

/// [R-MODULE-020] a replaced module is recorded as replaced and never
/// fetched, so a checkout somebody is editing does not have to be re-locked on
/// every save.
#[test]
fn a_replaced_module_is_locked_by_path_and_never_fetched() {
    let fs = tree();
    let offline = OfflineHttp;
    let synced = Syncer::new(&fs, &offline, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "0.2.17")],
            &[Replace {
                name: "stdlib".to_owned(),
                path: CHECKOUT.to_owned(),
                source: String::new(),
            }],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect("the replacement needs no network");

    let entry = synced.lock.modules.get("stdlib").expect("the entry");
    assert!(entry.replaced);
    assert_eq!(entry.path, CHECKOUT);
    assert!(entry.version.is_empty());
    assert_eq!(
        synced
            .replaced
            .get("stdlib")
            .map(|p| p.display().to_string()),
        Some(CHECKOUT.to_owned())
    );
}

/// [R-MODULE-064] a `replace` pointing at nothing is a typo, and falling back
/// to upstream would silently do the opposite of what it asked.
#[test]
fn a_replacement_that_is_not_there_fails_the_sync() {
    let fs = tree();
    let http = registry_http();
    let err = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "0.2.17")],
            &[Replace {
                name: "stdlib".to_owned(),
                path: "/nowhere".to_owned(),
                source: String::new(),
            }],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect_err("the missing checkout is reported");
    assert!(
        matches!(err, ModuleError::NoSuchReplacement { .. }),
        "{err}"
    );
    assert!(http.asked().is_empty(), "nothing was fetched");
}

/// [R-MODULE-021] a fork is remote input like anything else: the directive
/// says where to fetch, not that the code is trusted.
#[test]
fn a_remote_replacement_is_fetched_and_pinned_like_any_other_module() {
    let fs = tree();
    let archive = github_tarball(
        "stdlib-abc123",
        &[ArchiveEntry::file(
            "MODULE.meow",
            "module(name = \"stdlib\")\n",
        )],
    );
    let http = ScriptedHttp::new()
        .with(
            "https://api.github.com/repos/fork/stdlib/commits/v9",
            br#"{"sha":"abc123"}"#.to_vec(),
        )
        .with(
            "https://github.com/fork/stdlib/archive/abc123.tar.gz",
            archive,
        );

    let synced = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "0.2.17")],
            &[Replace {
                name: "stdlib".to_owned(),
                path: String::new(),
                source: "github:fork/stdlib@v9".to_owned(),
            }],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect("the fork resolves");

    let entry = synced.lock.modules.get("stdlib").expect("the entry");
    assert_eq!(entry.commit_sha, "abc123");
    assert!(entry.integrity.starts_with("sha384-"), "{entry:?}");
    assert!(
        !entry.replaced,
        "a remote fork is fetched, not replaced in place"
    );
}

/// [R-MODULE-012] an aggregate module brings its dependencies with it, read
/// from its own manifest.
#[test]
fn a_github_modules_transitive_dependencies_are_walked() {
    let fs = tree();
    let parent = github_tarball(
        "parent-p1",
        &[ArchiveEntry::file(
            "MODULE.meow",
            "module(name = \"parent\")\ndep(name = \"child\", source = \"github:o/child@v1\")\n",
        )],
    );
    let child = github_tarball(
        "child-c1",
        &[ArchiveEntry::file(
            "MODULE.meow",
            "module(name = \"child\")\n",
        )],
    );
    let http = ScriptedHttp::new()
        .with(
            "https://api.github.com/repos/o/parent/commits/v1",
            br#"{"sha":"p1"}"#.to_vec(),
        )
        .with("https://github.com/o/parent/archive/p1.tar.gz", parent)
        .with(
            "https://api.github.com/repos/o/child/commits/v1",
            br#"{"sha":"c1"}"#.to_vec(),
        )
        .with("https://github.com/o/child/archive/c1.tar.gz", child);

    let synced = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .sync(
            &[from_source("parent", "github:o/parent@v1")],
            &[],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect("the aggregate resolves");

    assert_eq!(
        synced
            .lock
            .modules
            .get("child")
            .map(|e| e.commit_sha.clone()),
        Some("c1".to_owned()),
        "the child came along: {:?}",
        synced.lock.modules.keys().collect::<Vec<_>>()
    );
}

/// [R-MODULE-001] the version a module's own manifest asks for is selected
/// too, which is what makes a lock cover the whole graph rather than the
/// names somebody typed.
#[test]
fn a_transitive_registry_dependency_is_selected_and_locked() {
    let stdlib_with_dep = tarball(&[ArchiveEntry::file(
        "MODULE.meow",
        "module(name = \"stdlib\", version = \"0.2.17\")\ndep(name = \"helper\", version = \"1.1.0\")\n",
    )]);
    let helper = helper();
    let index = format!(
        r#"compat = 1

[modules.stdlib]
versions = ["0.2.17"]
source = "{REGISTRY}/t/{{name}}-{{version_no_v}}.tar.gz"

[modules.helper]
versions = ["1.0.0", "1.1.0"]
source = "{REGISTRY}/t/{{name}}-{{version_no_v}}.tar.gz"
"#
    );
    let http = ScriptedHttp::new()
        .with(INDEX_URL, index.into_bytes())
        .with(
            format!("{REGISTRY}/t/stdlib-0.2.17.tar.gz"),
            stdlib_with_dep,
        )
        .with(format!("{REGISTRY}/t/helper-1.1.0.tar.gz"), helper);

    let fs = tree();
    let synced = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "0.2.17")],
            &[],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect("the graph resolves");

    assert_eq!(
        synced.resolved.get("helper").map(String::as_str),
        Some("1.1.0")
    );
}

/// [R-MODULE-022] a machine-local override exists precisely to differ from
/// what the configuration commits.
#[test]
fn a_local_override_wins_over_the_shared_one() {
    let shared = vec![
        Replace {
            name: "stdlib".to_owned(),
            path: "/shared".to_owned(),
            source: String::new(),
        },
        Replace {
            name: "other".to_owned(),
            path: "/kept".to_owned(),
            source: String::new(),
        },
    ];
    let local = vec![
        Replace {
            name: "stdlib".to_owned(),
            path: "/mine".to_owned(),
            source: String::new(),
        },
        Replace {
            name: "extra".to_owned(),
            path: "/added".to_owned(),
            source: String::new(),
        },
    ];

    let merged = overlay_replaces(&shared, &local);
    let by_name: BTreeMap<&str, &str> = merged
        .iter()
        .map(|r| (r.name.as_str(), r.path.as_str()))
        .collect();
    assert_eq!(by_name.get("stdlib"), Some(&"/mine"));
    assert_eq!(by_name.get("other"), Some(&"/kept"));
    assert_eq!(by_name.get("extra"), Some(&"/added"));
}

/// [R-MODULE-052] the two manifests produce two locks, and a module in both
/// resolves in each rather than one of them winning silently.
#[test]
fn each_manifest_produces_its_own_lock() {
    let fs = tree();
    let http = registry_http();
    let syncer =
        Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t").with_index_url(INDEX_URL);

    let shared = syncer
        .sync(
            &[dep("stdlib", "0.2.17")],
            &[],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect("the shared manifest syncs");
    let local = syncer
        .sync(
            &[dep("helper", "1.1.0")],
            &[],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect("the local manifest syncs");

    assert_eq!(shared.lock.modules.keys().collect::<Vec<_>>(), ["stdlib"]);
    assert_eq!(local.lock.modules.keys().collect::<Vec<_>>(), ["helper"]);
}

/// A sync whose every dependency is local or on GitHub must not fetch the
/// index, because a machine with no network still has to be able to run it.
#[test]
fn the_index_is_fetched_only_when_a_registry_module_needs_it() {
    let fs = tree();
    let offline = OfflineHttp;
    Syncer::new(&fs, &offline, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("forked", "0.1.0")],
            &[Replace {
                name: "forked".to_owned(),
                path: CHECKOUT.to_owned(),
                source: String::new(),
            }],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect("nothing needed the index");
}

/// [R-MODULE-061] a version the index does not publish is a different mistake
/// from a module it does not publish, and both are the user's to fix.
#[test]
fn a_version_the_index_does_not_publish_is_named() {
    let fs = tree();
    let http = registry_http();
    let err = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "9.9.9")],
            &[],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect_err("9.9.9 was never published");
    assert!(matches!(err, ModuleError::NoSuchVersion { .. }), "{err}");
}

/// A `dep()` with no version takes what the index published most recently,
/// which is what `latest` means and what an empty version meant in `v0.1.0`.
#[test]
fn an_unversioned_dependency_takes_the_latest_published() {
    let fs = tree();
    let http = registry_http();
    let synced = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "")],
            &[],
            &LockFile::default(),
            &Upgrade::Nothing,
        )
        .expect("the latest resolves");
    assert_eq!(
        synced.resolved.get("stdlib").map(String::as_str),
        Some("0.2.17")
    );
}

/// A sync must not throw away what other commands recorded: the packages an
/// apply installed and the GitHub files a load pinned are in the same file.
#[test]
fn a_sync_carries_the_other_tables_through() {
    let fs = tree();
    let http = registry_http();
    let previous = LockFile {
        packages: BTreeMap::from([(
            "brew".to_owned(),
            BTreeMap::from([(
                "git".to_owned(),
                meowctl_config::PackageEntry {
                    requested: "latest".to_owned(),
                    installed: "2.43.0".to_owned(),
                    note: String::new(),
                },
            )]),
        )]),
        github: BTreeMap::from([(
            "o/r@v1//init.star".to_owned(),
            meowctl_config::GitHubEntry {
                commit: "abc".to_owned(),
                integrity: "sha384-AAA".to_owned(),
            },
        )]),
        ..LockFile::default()
    };

    let synced = Syncer::new(&fs, &http, Cache::new("/cache"), "0.2.0", "t")
        .with_index_url(INDEX_URL)
        .sync(
            &[dep("stdlib", "0.2.17")],
            &[],
            &previous,
            &Upgrade::Nothing,
        )
        .expect("the manifest syncs");
    assert_eq!(synced.lock.packages, previous.packages);
    assert_eq!(synced.lock.github, previous.github);
}
