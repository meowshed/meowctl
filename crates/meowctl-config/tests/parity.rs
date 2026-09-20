//! Whether a file written here is the file `v0.1.0` would have written.
//!
//! The fixture is not hand-written: it came out of `lock.Write` in the Go
//! tree, given a structure with every field populated. A round trip through
//! our own reader and writer proves the two agree with each other; only a
//! fixture proves they agree with `v0.1.0`; see [R-CONFIG-023].

// `clippy.toml` exempts tests from `expect_used`, but only a function carrying
// `#[test]`. A helper in a test binary is test code by construction.
#![allow(clippy::expect_used)]

use std::collections::BTreeMap;
use std::path::Path;

use meowctl_config::{
    GitHubEntry, InstalledComponent, InstalledLock, LockFile, LockMeta, ModuleEntry, PackageEntry,
    Sentinel,
};
use meowctl_fs::{FileSystem, MemFs};

/// The lock `v0.1.0` wrote, checked into the tree.
const V0_1_0_LOCK: &str = include_str!("fixtures/deps.lock.v0_1_0");

fn memory() -> MemFs {
    let fs = MemFs::new();
    fs.create_dir_all(Path::new("/cfg")).expect("config dir");
    fs
}

/// The structure the fixture was generated from.
fn populated() -> LockFile {
    let mut modules = BTreeMap::new();
    modules.insert(
        "stdlib".to_owned(),
        ModuleEntry {
            version: "0.2.17".to_owned(),
            source: "https://example.invalid/stdlib.tar.gz".to_owned(),
            integrity: "sha384-AAA".to_owned(),
            files: [
                (
                    "components/zsh/init.star".to_owned(),
                    "sha384-BBB".to_owned(),
                ),
                ("MODULE.meow".to_owned(), "sha384-CCC".to_owned()),
            ]
            .into_iter()
            .collect(),
            ..ModuleEntry::default()
        },
    );
    modules.insert(
        "forked".to_owned(),
        ModuleEntry {
            replaced: true,
            path: "/local/checkout".to_owned(),
            ..ModuleEntry::default()
        },
    );
    modules.insert(
        "gh".to_owned(),
        ModuleEntry {
            version: "1.0.0".to_owned(),
            source: "github:o/r@v1".to_owned(),
            integrity: "sha384-DDD".to_owned(),
            commit_sha: "abc123".to_owned(),
            ..ModuleEntry::default()
        },
    );

    let mut github = BTreeMap::new();
    github.insert(
        "github.com/o/r".to_owned(),
        GitHubEntry {
            commit: "abc123".to_owned(),
            integrity: "sha384-EEE".to_owned(),
        },
    );

    let mut brew = BTreeMap::new();
    brew.insert(
        "git".to_owned(),
        PackageEntry {
            requested: "latest".to_owned(),
            installed: "2.43.0".to_owned(),
            note: String::new(),
        },
    );
    brew.insert(
        "jq".to_owned(),
        PackageEntry {
            requested: "1.7".to_owned(),
            installed: "1.7".to_owned(),
            note: "pinned".to_owned(),
        },
    );

    LockFile {
        meta: LockMeta {
            generated_by: "v0.1.0".to_owned(),
            updated_at: "2026-09-19T12:00:00Z".to_owned(),
        },
        modules,
        github,
        packages: [("brew".to_owned(), brew)].into_iter().collect(),
    }
}

/// [R-CONFIG-023] byte for byte, because a semantic comparison hides a key
/// reorder and a reorder is what breaks reproducibility.
///
/// The fixture is also where [R-CONFIG-020] and [R-CONFIG-021] are held: the
/// four tables and the per-file hash map are in it, so a table dropped or a
/// key renamed fails here rather than on somebody's machine.
#[test]
fn a_lock_file_is_written_exactly_as_v0_1_0_writes_it() {
    let fs = memory();
    let path = Path::new("/cfg/deps.lock");
    populated().write(&fs, path).expect("write");

    let written = String::from_utf8(fs.read(path).expect("read")).expect("utf8");
    assert_eq!(
        written, V0_1_0_LOCK,
        "the layout drifted from what v0.1.0 emits"
    );
}

/// The fixture is also a parser test: whatever `v0.1.0` writes has to come
/// back as the structure it came from.
#[test]
fn a_v0_1_0_lock_file_parses_back_to_its_structure() {
    let fs = memory();
    let path = Path::new("/cfg/deps.lock");
    fs.write(path, V0_1_0_LOCK.as_bytes()).expect("seed");

    assert_eq!(LockFile::read(&fs, path).expect("read"), populated());
}

/// [R-CONFIG-003] a first run has no lock, and treating that as a failure
/// would make `init` the only command that works.
#[test]
fn a_missing_lock_is_an_empty_one() {
    let fs = memory();
    let lock = LockFile::read(&fs, Path::new("/cfg/absent.lock")).expect("read");
    assert_eq!(lock, LockFile::default());
}

/// [R-CONFIG-024] the local file is how a machine differs from the committed
/// configuration, so it wins.
#[test]
fn a_local_lock_overlays_the_shared_one() {
    let mut shared = LockFile::default();
    shared.modules.insert(
        "stdlib".to_owned(),
        ModuleEntry {
            version: "1.0.0".to_owned(),
            ..ModuleEntry::default()
        },
    );
    shared.modules.insert(
        "other".to_owned(),
        ModuleEntry {
            version: "9.9.9".to_owned(),
            ..ModuleEntry::default()
        },
    );

    let mut local = LockFile::default();
    local.modules.insert(
        "stdlib".to_owned(),
        ModuleEntry {
            version: "2.0.0".to_owned(),
            ..ModuleEntry::default()
        },
    );

    let merged = shared.overlaid_with(&local);
    assert_eq!(merged.modules["stdlib"].version, "2.0.0");
    assert_eq!(merged.modules["other"].version, "9.9.9");
}

/// [R-CONFIG-031] a user who installed an earlier build has the version 1
/// shape on disk, and a parse error would strand them.
#[test]
fn the_version_1_installed_lock_still_parses() {
    let fs = memory();
    let path = Path::new("/cfg/installed.lock");
    fs.write(
        path,
        b"schema_version = 1\ncomponents = [\"neovim\", \"zsh\"]\n",
    )
    .expect("seed");

    let lock = InstalledLock::read(&fs, path).expect("read");
    assert_eq!(lock.names(), ["neovim", "zsh"]);

    // Every version is unknown in the old shape, which is what
    // `installedLock.versionMap` reports.
    let fingerprints = lock.fingerprints();
    assert_eq!(fingerprints["neovim"], "");
    assert_eq!(fingerprints["zsh"], "");
}

/// [R-CONFIG-030] and [R-CONFIG-032]: each component with the fingerprint of
/// the module it came from, sorted, so the file does not churn and shows a
/// diff on every
/// apply.
#[test]
fn an_installed_lock_is_written_sorted_and_in_the_current_shape() {
    let fs = memory();
    let path = Path::new("/cfg/installed.lock");

    InstalledLock::write(
        &fs,
        path,
        &[
            InstalledComponent {
                name: "zsh".to_owned(),
                version: "1.0".to_owned(),
            },
            InstalledComponent {
                name: "@dotmeow".to_owned(),
                version: String::new(),
            },
        ],
    )
    .expect("write");

    let written = String::from_utf8(fs.read(path).expect("read")).expect("utf8");
    assert_eq!(
        written,
        "schema_version = 2\n\n[[installed]]\n  name = \"@dotmeow\"\n[[installed]]\n  name = \"zsh\"\n  version = \"1.0\"\n"
    );
}

/// [R-CONFIG-041] `v0.1.0` ignores the field, so an older binary rewrites a
/// newer file and drops what it did not understand.
#[test]
fn a_sentinel_from_a_newer_build_is_refused_rather_than_overwritten() {
    let fs = memory();
    let path = Path::new("/cfg/state.toml");
    fs.write(path, b"schema_version = 99\n").expect("seed");

    let err = Sentinel::read(&fs, path).expect_err("should refuse");
    let message = err.to_string();
    assert!(message.contains("99"), "{message}");
    assert!(message.contains("newer"), "{message}");
}

/// [R-CONFIG-040] the sentinel is read back as what was written, including the
/// timestamps, which are TOML datetimes rather than strings.
///
/// The fixture carries what [R-CONFIG-042], [R-CONFIG-043] and
/// [R-CONFIG-044] require: `last_run` with its four fields, a
/// `completed_components` entry with its phase, component and timestamp, and
/// `rolled_back` as one of the four strings.
#[test]
fn a_sentinel_round_trips() {
    let fs = memory();
    let path = Path::new("/cfg/state.toml");
    fs.write(
        path,
        concat!(
            "schema_version = 1\n",
            "repo_url = \"https://example.invalid/dotfiles\"\n",
            "\n",
            "[last_run]\n",
            "  phase_set = \"install\"\n",
            "  started_at = 2026-09-19T14:38:45.972634Z\n",
            "  completed = true\n",
            "  rolled_back = \"\"\n",
            "\n",
            "[[completed_components]]\n",
            "  phase = \"install\"\n",
            "  component = \"neovim\"\n",
            "  completed_at = 2026-09-19T14:38:46Z\n",
        )
        .as_bytes(),
    )
    .expect("seed");

    let sentinel = Sentinel::read(&fs, path).expect("read");
    assert!(sentinel.is_completed("install", "neovim"));
    assert!(!sentinel.is_completed("verify", "neovim"));

    let out = Path::new("/cfg/written.toml");
    sentinel.write(&fs, out).expect("write");
    assert_eq!(Sentinel::read(&fs, out).expect("reread"), sentinel);
}

/// [R-ENGINE-043] a module bump has to make the next run redo the components
/// that came from it.
#[test]
fn forgetting_a_component_clears_every_phase_it_completed() {
    let fs = memory();
    let path = Path::new("/cfg/state.toml");
    fs.write(
        path,
        concat!(
            "schema_version = 1\n\n",
            "[[completed_components]]\n  phase = \"install\"\n  component = \"a\"\n",
            "[[completed_components]]\n  phase = \"install_configure\"\n  component = \"a\"\n",
            "[[completed_components]]\n  phase = \"install\"\n  component = \"b\"\n",
        )
        .as_bytes(),
    )
    .expect("seed");

    let mut sentinel = Sentinel::read(&fs, path).expect("read");
    sentinel.forget("a");

    assert!(!sentinel.is_completed("install", "a"));
    assert!(!sentinel.is_completed("install_configure", "a"));
    assert!(sentinel.is_completed("install", "b"));
}

/// [R-CONFIG-001] these are the names a user has on disk, so they are not ours
/// to change.
#[test]
fn the_file_names_are_the_ones_v0_1_0_uses() {
    let layout = meowctl_config::Layout::new("/cfg");
    let name = |p: std::path::PathBuf| {
        p.file_name()
            .expect("a file name")
            .to_string_lossy()
            .into_owned()
    };

    assert_eq!(name(layout.entry()), "init.star");
    assert_eq!(name(layout.local_entry()), "local.star");
    assert_eq!(name(layout.modfile()), "deps.mod");
    assert_eq!(name(layout.local_modfile()), "deps.local.mod");
    assert_eq!(name(layout.lock()), "deps.lock");
    assert_eq!(name(layout.local_lock()), "deps.local.lock");
    assert_eq!(name(layout.packages_lock()), "pkgs.lock");
    assert_eq!(name(layout.local_packages_lock()), "pkgs.local.lock");
    assert_eq!(name(layout.state()), "state.toml");
    assert_eq!(name(layout.installed()), "installed.lock");
}

/// [R-CONFIG-004] a user who wrote their configuration before the rename needs
/// the three commands that fix it, not a missing-file error.
#[test]
fn a_pre_rename_directory_is_told_how_to_migrate() {
    let fs = memory();
    fs.write(Path::new("/cfg/meowctl.star"), b"# old\n")
        .expect("seed");

    let layout = meowctl_config::Layout::new("/cfg");
    let err = layout.check(&fs).expect_err("should refuse");
    let message = err.to_string();

    // Built rather than written out: a path separator is the platform's, and
    // hard-coding one checks the test's spelling instead of the message.
    let shown = |p: std::path::PathBuf| p.display().to_string();
    let root = Path::new("/cfg");
    for (from, to) in [
        (shown(root.join("meowctl.star")), shown(layout.entry())),
        (shown(root.join("meowctl.mod")), shown(layout.modfile())),
        (shown(root.join("meowctl.lock")), shown(layout.lock())),
    ] {
        assert!(message.contains(&format!("mv {from} {to}")), "{message}");
    }
}

/// [R-CLI-050] someone who has not run `init` should be told to, not handed a
/// path they have never seen.
#[test]
fn a_directory_with_no_configuration_says_to_run_init() {
    let fs = memory();
    let layout = meowctl_config::Layout::new("/cfg");

    let err = layout.check(&fs).expect_err("should refuse");
    assert!(err.to_string().contains("meowctl init"), "{err}");
}

#[test]
fn a_configured_directory_passes() {
    let fs = memory();
    fs.write(Path::new("/cfg/init.star"), b"# hi\n")
        .expect("seed");
    meowctl_config::Layout::new("/cfg")
        .check(&fs)
        .expect("check");
}

/// The manifest `v0.1.0` wrote, checked into the tree.
const V0_1_0_MODFILE: &str = include_str!("fixtures/deps.mod.v0_1_0");

/// [R-CONFIG-011], [R-CONFIG-013] and [R-CONFIG-014]: byte for byte, from a
/// fixture `modfile.Write` produced rather than from a reading of it. The
/// four statements are all in it, and so is the keyword order.
#[test]
fn a_manifest_is_written_exactly_as_v0_1_0_writes_it() {
    use meowctl_config::{Dep, Modfile, Module, Replace};

    let modfile = Modfile {
        module: Some(Module {
            name: "my-dotfiles".to_owned(),
            version: "0.1.0".to_owned(),
        }),
        deps: vec![
            Dep {
                name: "stdlib".to_owned(),
                version: "0.2.17".to_owned(),
                source: String::new(),
            },
            Dep {
                name: "plug".to_owned(),
                version: String::new(),
                source: "github:o/r@v1".to_owned(),
            },
        ],
        replaces: vec![
            Replace {
                name: "stdlib".to_owned(),
                path: "/local/checkout".to_owned(),
                source: String::new(),
            },
            Replace {
                name: "plug".to_owned(),
                path: String::new(),
                source: "github:fork/r@v2".to_owned(),
            },
        ],
    };

    assert_eq!(
        modfile.render(),
        V0_1_0_MODFILE,
        "the manifest layout drifted from what v0.1.0 emits"
    );
}

/// [R-CONFIG-010] a manifest this writer produced has to evaluate, or
/// `meowctl dep add` writes a file the next run cannot read.
#[test]
fn a_written_manifest_evaluates() {
    use meowctl_starlark::{Evaluator, NoLoader, Platform};

    let loader = NoLoader;
    let evaluator = Evaluator::new(Platform::current(), &loader);
    let result = evaluator
        .evaluate("deps.mod", V0_1_0_MODFILE)
        .expect("the manifest should evaluate");

    let module = result.declarations.module.expect("a module declaration");
    assert_eq!(module.name, "my-dotfiles");
    assert_eq!(result.declarations.deps.len(), 2);
    assert_eq!(result.declarations.replaces.len(), 2);
}

/// [R-CONFIG-061] a hash that is not a W3C Subresource Integrity one fails
/// where the file is read.
///
/// The alternative is what the first implementation did: keep the string,
/// compare it against a computed hash later, and report a mismatch. A user
/// then sees "the tarball does not match what the lock records", which is how
/// tampering looks, for a lock somebody hand-edited.
#[test]
fn a_lock_with_a_malformed_integrity_is_refused() {
    let fs = memory();
    let path = Path::new("/cfg/deps.lock");
    fs.write(
        path,
        concat!(
            "[modules]\n",
            "  [modules.stdlib]\n",
            "    version = \"0.2.17\"\n",
            "    source = \"https://r/t.tar.gz\"\n",
            "    integrity = \"not-a-hash\"\n",
        )
        .as_bytes(),
    )
    .expect("seed");

    let err = LockFile::read(&fs, path).expect_err("a malformed hash should be refused");
    let said = err.to_string();
    assert!(said.contains("stdlib"), "{said}");
    assert!(said.contains("sha384-"), "{said}");
}

/// [R-CONFIG-061] and a per-file hash is held to the same form, because that
/// is what [R-MODULE-041] compares a cached module against.
#[test]
fn a_malformed_per_file_hash_is_refused() {
    let fs = memory();
    let path = Path::new("/cfg/deps.lock");
    fs.write(
        path,
        concat!(
            "[modules]\n",
            "  [modules.stdlib]\n",
            "    version = \"0.2.17\"\n",
            "    integrity = \"sha384-AAA\"\n",
            "    [modules.stdlib.files]\n",
            "      \"components/git.star\" = \"md5-nope\"\n",
        )
        .as_bytes(),
    )
    .expect("seed");

    let err = LockFile::read(&fs, path).expect_err("a malformed per-file hash should be refused");
    assert!(err.to_string().contains("components/git.star"), "{err}");
}

/// [R-CONFIG-061] an empty hash is not malformed. A replaced module points at
/// a local path and has nothing to verify.
#[test]
fn a_replaced_module_with_no_hash_is_accepted() {
    let fs = memory();
    let path = Path::new("/cfg/deps.lock");
    fs.write(
        path,
        concat!(
            "[modules]\n",
            "  [modules.stdlib]\n",
            "    replaced = true\n",
            "    path = \"../meowctl-stdlib\"\n",
        )
        .as_bytes(),
    )
    .expect("seed");

    let lock = LockFile::read(&fs, path).expect("a replaced module has no hash to check");
    assert!(lock.modules["stdlib"].replaced);
}

/// [R-CONFIG-012] a dependency naming both leaves the resolver choosing
/// between two answers, so reading the manifest fails instead.
#[test]
fn a_dep_with_a_version_and_a_source_is_refused() {
    use meowctl_config::{Dep, Modfile};

    let modfile = Modfile {
        deps: vec![Dep {
            name: "stdlib".to_owned(),
            version: "0.2.17".to_owned(),
            source: "github:o/r@v1".to_owned(),
        }],
        ..Modfile::default()
    };

    let err = modfile
        .check(Path::new("/cfg/deps.mod"))
        .expect_err("both fields should be refused");
    let said = err.to_string();
    assert!(said.contains("stdlib"), "{said}");
    assert!(said.contains("mutually exclusive"), "{said}");
}

/// [R-CONFIG-012] and one naming neither is equally unusable.
#[test]
fn a_dep_with_neither_is_refused() {
    use meowctl_config::{Dep, Modfile};

    let modfile = Modfile {
        deps: vec![Dep {
            name: "stdlib".to_owned(),
            version: String::new(),
            source: String::new(),
        }],
        ..Modfile::default()
    };

    let err = modfile
        .check(Path::new("/cfg/deps.mod"))
        .expect_err("neither field should be refused");
    assert!(err.to_string().contains("exactly one"), "{err}");
}

/// [R-CONFIG-012] `replace()` carries the same rule, with its own two fields.
#[test]
fn a_replace_follows_the_same_rule() {
    use meowctl_config::{Modfile, Replace};

    let both = Modfile {
        replaces: vec![Replace {
            name: "stdlib".to_owned(),
            path: "/local".to_owned(),
            source: "github:fork/r@v2".to_owned(),
        }],
        ..Modfile::default()
    };
    assert!(both.check(Path::new("/cfg/deps.mod")).is_err());

    let neither = Modfile {
        replaces: vec![Replace {
            name: "stdlib".to_owned(),
            path: String::new(),
            source: String::new(),
        }],
        ..Modfile::default()
    };
    let err = neither
        .check(Path::new("/cfg/deps.mod"))
        .expect_err("neither field should be refused");
    assert!(err.to_string().contains("path"), "{err}");
}

/// [R-CONFIG-011] and [R-CONFIG-012]: the four statements a manifest may
/// carry, each in a form the rule accepts.
#[test]
fn the_four_statements_a_manifest_carries_are_accepted() {
    use meowctl_config::{Dep, Modfile, Module, Replace};

    let modfile = Modfile {
        module: Some(Module {
            name: "my-dotfiles".to_owned(),
            version: "0.1.0".to_owned(),
        }),
        deps: vec![
            Dep {
                name: "stdlib".to_owned(),
                version: "0.2.17".to_owned(),
                source: String::new(),
            },
            Dep {
                name: "plug".to_owned(),
                version: String::new(),
                source: "github:o/r@v1".to_owned(),
            },
        ],
        replaces: vec![
            Replace {
                name: "stdlib".to_owned(),
                path: "/local/checkout".to_owned(),
                source: String::new(),
            },
            Replace {
                name: "plug".to_owned(),
                path: String::new(),
                source: "github:fork/r@v2".to_owned(),
            },
        ],
    };

    modfile
        .check(Path::new("/cfg/deps.mod"))
        .expect("the four statements are what a manifest is made of");
}

/// [R-CONFIG-060] the parser's position reaches the user, because "this file
/// is malformed" for a 200-line lock sends them reading the whole thing.
#[test]
fn a_malformed_file_names_the_file_and_where_it_broke() {
    let fs = memory();
    let path = Path::new("/cfg/deps.lock");
    fs.write(
        path,
        b"[modules]\n  [modules.stdlib]\n    version = \"unterminated\n",
    )
    .expect("seed");

    let err = LockFile::read(&fs, path).expect_err("malformed TOML should be refused");
    let said = err.to_string();
    assert!(said.contains("deps.lock"), "the file is not named: {said}");
    assert!(
        said.contains("line 3") || said.contains("3:"),
        "the position is not in the message: {said}"
    );
}

/// [R-CONFIG-033] the fingerprint is a chain, not one field: a GitHub module
/// re-synced to a new commit has to invalidate even though no version moved.
#[test]
fn a_fingerprint_falls_through_version_then_commit_then_hash() {
    use meowctl_config::ModuleEntry;

    let registry = ModuleEntry {
        version: "0.2.17".to_owned(),
        commit_sha: "abc".to_owned(),
        integrity: "sha384-AAA".to_owned(),
        ..ModuleEntry::default()
    };
    assert_eq!(registry.fingerprint(), "0.2.17");

    let github = ModuleEntry {
        commit_sha: "abc".to_owned(),
        integrity: "sha384-AAA".to_owned(),
        ..ModuleEntry::default()
    };
    assert_eq!(github.fingerprint(), "abc");

    let hashed = ModuleEntry {
        integrity: "sha384-AAA".to_owned(),
        ..ModuleEntry::default()
    };
    assert_eq!(hashed.fingerprint(), "sha384-AAA");
}
