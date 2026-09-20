//! What has been done, and what has to be done again.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

mod support;

use std::collections::BTreeMap;
use std::path::Path;

use meowctl_common::{ComponentId, Outcome, PhaseSet};
use meowctl_config::{LockFile, ModuleEntry, Sentinel};
use meowctl_engine::{
    Declaration, Graph, Inputs, Plan, Progress, Runner, discover, fingerprints, interrupted_run,
    stale_components,
};
use meowctl_fs::FileSystem as _;
use meowctl_starlark::Evaluator;
use support::{Config, NoLoads, macos, settings, world};

fn ids(names: &[&str]) -> Vec<ComponentId> {
    names.iter().map(|n| n.parse().expect(n)).collect()
}

fn recorded(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs
        .iter()
        .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
        .collect()
}

/// [R-ENGINE-041] a component is recorded as soon as its hook succeeds, so a
/// run interrupted at the next one resumes here rather than at the start of
/// the phase. `v0.1.0` records the whole order once the phase is clean, which
/// loses everything that phase had done.
#[test]
fn a_component_is_recorded_as_soon_as_it_succeeds() {
    let config = Config::new()
        .declaring(
            Declaration::new("first"),
            "component(\"first\")\ndef install(ctx):\n    ctx.log(\"ok\")\n",
        )
        .declaring(
            Declaration::new("second").after(&["first"]),
            "component(\"second\")\ndef install(ctx):\n    fail(\"no\")\n",
        );

    let loader = NoLoads;
    let platform = macos();
    let discovered = discover(&config, &loader, &platform).expect("discovery");
    let graph = Graph::build(&discovered).expect("the graph builds");
    let plan = Plan::compute(&graph, PhaseSet::Install, &Inputs::default());

    let (effects, world) = world(Vec::new(), None);
    let state = Path::new(support::HOME).join("state.toml");
    let evaluator = Evaluator::new(platform.clone(), &loader);
    let mut runner = Runner::new(
        &graph,
        &discovered.registry,
        evaluator,
        effects,
        settings(&platform, false),
    )
    .recording(Progress::new(Sentinel::default(), &state));

    let report = runner.run(&plan);
    assert!(report.failure.is_some(), "the second component failed");

    let written = runner.progress().expect("progress").sentinel();
    assert!(
        written.is_completed("install", "first"),
        "the one that succeeded is recorded: {written:?}"
    );
    assert!(
        !written.is_completed("install", "second"),
        "the one that failed is not"
    );
    assert!(
        world.fs.exists(&state).expect("asking"),
        "and it is on disk, not only in memory"
    );
}

/// [R-ENGINE-033] a component with nothing to do in a phase is recorded as
/// having done it: absence means it has nothing to do, which is a success,
/// and re-running it every time would be work with no result.
#[test]
fn a_component_with_nothing_to_do_is_still_recorded() {
    let config = Config::new().declaring(
        Declaration::new("zsh"),
        "component(\"zsh\")\ndef install(ctx):\n    ctx.log(\"ok\")\n",
    );

    let loader = NoLoads;
    let platform = macos();
    let discovered = discover(&config, &loader, &platform).expect("discovery");
    let graph = Graph::build(&discovered).expect("the graph builds");
    let plan = Plan::compute(&graph, PhaseSet::Install, &Inputs::default());

    let (effects, _) = world(Vec::new(), None);
    let evaluator = Evaluator::new(platform.clone(), &loader);
    let mut runner = Runner::new(
        &graph,
        &discovered.registry,
        evaluator,
        effects,
        settings(&platform, false),
    )
    .recording(Progress::new(
        Sentinel::default(),
        Path::new(support::HOME).join("state.toml"),
    ));
    let report = runner.run(&plan);

    assert!(report.succeeded(), "{report:?}");
    let written = runner.progress().expect("progress").sentinel();
    for phase in ["install_check", "install", "install_configure"] {
        assert!(
            written.is_completed(phase, "zsh"),
            "{phase} is recorded even where the hook was absent"
        );
    }
    assert!(
        report
            .finished
            .iter()
            .any(|(_, _, outcome)| *outcome == Outcome::NothingToDo),
        "and it is reported as having had nothing to do"
    );
}

/// [R-ENGINE-040] a recorded component is skipped on the next run, which is
/// the whole point of recording it.
#[test]
fn what_was_recorded_is_skipped_next_time() {
    let config = Config::new().declaring(
        Declaration::new("zsh"),
        "component(\"zsh\")\ndef install(ctx):\n    ctx.log(\"ok\")\n",
    );
    let loader = NoLoads;
    let platform = macos();
    let discovered = discover(&config, &loader, &platform).expect("discovery");
    let graph = Graph::build(&discovered).expect("the graph builds");

    let mut sentinel = Sentinel::default();
    sentinel.record("install", "zsh", None);
    let plan = Plan::compute(
        &graph,
        PhaseSet::Install,
        &Inputs {
            sentinel: Some(&sentinel),
            ..Inputs::default()
        },
    );
    assert_eq!(plan.running_in(meowctl_common::Phase::Install), 0);
}

/// [R-ENGINE-043] a bumped module clears every component that came from it,
/// including the transitive ones `installed.lock` does not name. Without this
/// a plain apply reported them already installed and left the symlinks
/// pointing at the old cached version.
#[test]
fn a_changed_module_makes_all_of_its_components_stale() {
    let components = ids(&[
        "@stdlib//components/zsh",
        "@stdlib//components/neovim",
        "@dotmeow//components/git",
        "local-thing",
    ]);
    let was = recorded(&[("zsh", "0.2.16"), ("neovim", "0.2.16"), ("git", "0.3.27")]);
    let now = recorded(&[("zsh", "0.2.17"), ("neovim", "0.2.17"), ("git", "0.3.27")]);

    let stale = stale_components(&components, &was, &now);
    assert_eq!(
        stale,
        ["zsh", "neovim"],
        "every component of the changed module, and nothing from the other"
    );
}

/// [R-ENGINE-043] and a component the recorded file names but that the
/// configuration no longer resolves is not stale: it is gone, and removing it
/// is a different command.
#[test]
fn a_component_that_no_longer_resolves_is_not_stale() {
    let components = ids(&["@stdlib//components/zsh"]);
    let was = recorded(&[("zsh", "0.2.17"), ("removed", "1.0.0")]);
    let now = recorded(&[("zsh", "0.2.17")]);
    assert!(stale_components(&components, &was, &now).is_empty());
}

/// [R-ENGINE-044] the fingerprint is the version, then the commit, then the
/// hash, so a GitHub module re-synced to a new commit invalidates even though
/// no version changed.
#[test]
fn the_fingerprint_falls_back_from_version_to_commit_to_hash() {
    let lock = LockFile {
        modules: BTreeMap::from([
            (
                "stdlib".to_owned(),
                ModuleEntry {
                    version: "0.2.17".to_owned(),
                    commit_sha: "abc".to_owned(),
                    integrity: "sha384-A".to_owned(),
                    ..ModuleEntry::default()
                },
            ),
            (
                "github.com/o/r".to_owned(),
                ModuleEntry {
                    commit_sha: "deadbeef".to_owned(),
                    integrity: "sha384-B".to_owned(),
                    ..ModuleEntry::default()
                },
            ),
            (
                "hashed".to_owned(),
                ModuleEntry {
                    integrity: "sha384-C".to_owned(),
                    ..ModuleEntry::default()
                },
            ),
        ]),
        ..LockFile::default()
    };

    let components = ids(&[
        "@stdlib//components/zsh",
        "github.com/o/r//components/tool",
        "@hashed//components/thing",
        "bare",
    ]);
    let found = fingerprints(&components, &lock);

    assert_eq!(found.get("zsh").map(String::as_str), Some("0.2.17"));
    assert_eq!(found.get("tool").map(String::as_str), Some("deadbeef"));
    assert_eq!(found.get("thing").map(String::as_str), Some("sha384-C"));
    assert!(
        !found.contains_key("bare"),
        "a component with no module has nothing that can go stale"
    );
}

/// [R-ENGINE-043] clearing a stale component throws away every phase it had
/// recorded, so the next run redoes the whole component rather than the part
/// of it that happened to be stale.
#[test]
fn clearing_a_stale_component_forgets_every_phase() {
    let (effects, world) = world(Vec::new(), None);
    let mut sentinel = Sentinel::default();
    sentinel.record("install_check", "zsh", None);
    sentinel.record("install", "zsh", None);
    sentinel.record("install", "neovim", None);

    let state = Path::new(support::HOME).join("state.toml");
    let mut progress = Progress::new(sentinel, &state);
    progress
        .forget(effects.fs.as_ref(), &["zsh".to_owned()])
        .expect("it writes");

    assert!(!progress.sentinel().is_completed("install", "zsh"));
    assert!(!progress.sentinel().is_completed("install_check", "zsh"));
    assert!(
        progress.sentinel().is_completed("install", "neovim"),
        "and leaves the others alone"
    );
    assert!(world.fs.exists(&state).expect("asking"));
}

/// [R-ENGINE-042] a journal left behind means the last run stopped partway
/// and the machine is in a state nobody chose, so it is something to report
/// before anything else happens.
#[test]
fn a_journal_left_behind_is_reported_as_an_interrupted_run() {
    let temp = tempfile::tempdir().expect("a temporary directory");
    let path = temp.path().join("journal.ndjson");

    assert_eq!(
        interrupted_run(&path),
        None,
        "nothing there is nothing to say"
    );

    std::fs::write(
        &path,
        "{\"seq\":1,\"phase\":\"install\",\"component\":\"zsh\",\"kind\":\"write_file\",\"inverse\":{}}\n",
    )
    .expect("writing a journal");
    assert_eq!(interrupted_run(&path), Some(1));
}
