//! Running a plan, against an in-memory world.
//!
//! Everything here runs real Starlark hooks against a `MemFs` and a scripted
//! executor, which is what the effects-behind-traits design buys: a test of
//! the runner needs no machine to run on.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

mod support;

use meowctl_common::{Event, Outcome, PhaseSet};
use meowctl_engine::{Declaration, Graph, Inputs, Plan, Runner, discover};
use meowctl_exec::ScriptedRun;
use meowctl_starlark::Evaluator;
use support::{Config, NoLoads, macos, settings, world};

/// A component whose install hook writes a file.
fn writes(name: &str, path: &str) -> String {
    format!("component({name:?})\ndef install(ctx):\n    ctx.write_file({path:?}, \"x\")\n")
}

/// A component whose install hook fails.
fn fails(name: &str) -> String {
    format!("component({name:?})\ndef install(ctx):\n    fail(\"no\")\n")
}

/// Runs a configuration and returns the report and the world it ran against.
fn run(
    config: &Config,
    runs: Vec<ScriptedRun>,
    rollback: bool,
    journal: Option<std::path::PathBuf>,
) -> (meowctl_engine::Report, support::World) {
    let loader = NoLoads;
    let platform = macos();
    let discovered = discover(config, &loader, &platform).expect("discovery");
    let graph = Graph::build(&discovered).expect("the graph builds");
    let plan = Plan::compute(&graph, PhaseSet::Install, &Inputs::default());

    let (effects, world) = world(runs, journal);
    let evaluator = Evaluator::new(platform.clone(), &loader);
    let mut runner = Runner::new(
        &graph,
        &discovered.registry,
        evaluator,
        effects,
        settings(&platform, rollback),
    );
    (runner.run(&plan), world)
}

/// [R-ENGINE-050] the stream a sink renders: a plan, then a phase around its
/// components, and a start and a finish around each.
#[test]
fn the_run_emits_the_events_a_sink_renders() {
    let config = Config::new().declaring(Declaration::new("zsh"), &writes("zsh", "~/.zshrc"));
    let (report, world) = run(&config, Vec::new(), false, None);
    assert!(report.succeeded(), "{report:?}");

    let kinds: Vec<&str> = world
        .seen()
        .iter()
        .map(|event| match event {
            Event::PlanComputed { .. } => "plan",
            Event::PhaseStarted { .. } => "phase-started",
            Event::PhaseFinished { .. } => "phase-finished",
            Event::ComponentStarted { .. } => "component-started",
            Event::ComponentFinished { .. } => "component-finished",
            Event::OpApplied { .. } => "op",
            _ => "other",
        })
        .collect();
    assert_eq!(kinds[0], "plan");
    assert!(
        kinds.contains(&"component-started") && kinds.contains(&"component-finished"),
        "{kinds:?}"
    );
    assert_eq!(
        kinds.iter().filter(|k| **k == "phase-started").count(),
        3,
        "one per phase in the install set: {kinds:?}"
    );
}

/// [R-ENGINE-033] an absent hook is a success with nothing to do, not a skip:
/// most components define three of the thirteen phases.
#[test]
fn a_component_with_no_hook_for_the_phase_has_nothing_to_do() {
    let config = Config::new().declaring(Declaration::new("zsh"), &writes("zsh", "~/.zshrc"));
    let (_, world) = run(&config, Vec::new(), false, None);

    let outcomes: Vec<Outcome> = world.finished().into_iter().map(|(_, o)| o).collect();
    assert_eq!(
        outcomes,
        [
            Outcome::NothingToDo,
            Outcome::Succeeded,
            Outcome::NothingToDo
        ],
        "install_check and install_configure are absent; install ran"
    );
}

/// [R-ENGINE-030] a phase stops at the first component that fails. Running on
/// would produce a cascade of errors about tools that never installed.
#[test]
fn a_phase_stops_at_the_first_component_that_fails() {
    let config = Config::new()
        .declaring(Declaration::new("first"), &fails("first"))
        .declaring(Declaration::new("second"), &writes("second", "~/second"));

    let (report, world) = run(&config, Vec::new(), false, None);
    let failure = report.failure.expect("it failed");
    assert_eq!(failure.component.as_str(), "first");

    assert!(
        !world.fs.paths().iter().any(|p| p.ends_with("second")),
        "the second component never ran: {:?}",
        world.fs.paths()
    );
}

/// [R-ENGINE-060] and the failure names the component, the phase, and what
/// went wrong.
#[test]
fn a_failure_names_the_component_the_phase_and_the_error() {
    let config = Config::new().declaring(Declaration::new("broken"), &fails("broken"));
    let (report, _) = run(&config, Vec::new(), false, None);

    let failure = report.failure.expect("it failed");
    let message = failure.to_string();
    assert!(message.contains("broken"), "{message}");
    assert!(message.contains("install"), "{message}");
    assert!(message.contains("no"), "{message}");
}

/// [R-ENGINE-031] a phase set stops at the first failed phase, so a component
/// is not configured after its install failed.
#[test]
fn a_phase_set_stops_at_the_first_failed_phase() {
    let config = Config::new().declaring(
        Declaration::new("zsh"),
        "component(\"zsh\")\ndef install(ctx):\n    fail(\"no\")\ndef install_configure(ctx):\n    ctx.write_file(\"~/configured\", \"x\")\n",
    );
    let (report, world) = run(&config, Vec::new(), false, None);

    assert!(report.failure.is_some());
    assert!(
        !world.fs.paths().iter().any(|p| p.ends_with("configured")),
        "install_configure never ran: {:?}",
        world.fs.paths()
    );
}

/// [R-ENGINE-063] a component that fails does not stop the run reporting the
/// ones that already succeeded.
#[test]
fn the_components_that_succeeded_are_still_reported() {
    let config = Config::new()
        .declaring(Declaration::new("first"), &writes("first", "~/first"))
        .declaring(
            Declaration::new("second").after(&["first"]),
            &fails("second"),
        );

    let (report, _) = run(&config, Vec::new(), false, None);
    let succeeded: Vec<&str> = report
        .finished
        .iter()
        .filter(|(_, _, outcome)| *outcome == Outcome::Succeeded)
        .map(|(_, id, _)| id.as_str())
        .collect();
    assert_eq!(succeeded, ["first"]);
    assert!(report.failure.is_some());
}

/// [R-ENGINE-032] a failed run undoes what it did, and says how that went.
#[test]
fn a_failed_run_rolls_back_what_it_did() {
    let temp = tempfile::tempdir().expect("a temporary directory");
    let config = Config::new()
        .declaring(Declaration::new("first"), &writes("first", "~/first"))
        .declaring(
            Declaration::new("second").after(&["first"]),
            &fails("second"),
        );

    let (report, world) = run(
        &config,
        Vec::new(),
        true,
        Some(temp.path().join("journal.ndjson")),
    );

    assert!(report.failure.is_some());
    let rolled_back = report.rolled_back.expect("a rollback ran");
    assert_eq!(rolled_back.outcome, meowctl_ops::Outcome::Ok);
    assert!(rolled_back.applied >= 1, "{rolled_back:?}");
    assert!(
        !world.fs.paths().iter().any(|p| p.ends_with("first")),
        "the write was undone: {:?}",
        world.fs.paths()
    );
}

/// [R-ENGINE-032] and a caller that turned rollback off keeps what the run
/// did, which is what `--no-rollback` is for.
#[test]
fn rollback_can_be_turned_off() {
    let temp = tempfile::tempdir().expect("a temporary directory");
    let config = Config::new()
        .declaring(Declaration::new("first"), &writes("first", "~/first"))
        .declaring(
            Declaration::new("second").after(&["first"]),
            &fails("second"),
        );

    let (report, world) = run(
        &config,
        Vec::new(),
        false,
        Some(temp.path().join("journal.ndjson")),
    );
    assert!(report.rolled_back.is_none());
    assert!(
        world.fs.paths().iter().any(|p| p.ends_with("first")),
        "what it did is still there: {:?}",
        world.fs.paths()
    );
}

/// [R-PM-015] and [R-PM-010]: a package a hook declares reaches the handler
/// for its manager, with the arguments the declaration carried.
#[test]
fn a_package_a_hook_declares_reaches_its_handler() {
    let handler = r#"
pm_name = "brew"
component("homebrew")
def install_pkg(ctx, name, version, **kwargs):
    ctx.run("brew", ["install", name])
def uninstall_pkg(ctx, name, version, **kwargs):
    pass
def interrogate(ctx):
    return []
"#;
    let config = Config::new()
        .declaring(Declaration::new("homebrew"), handler)
        .declaring(
            Declaration::new("tools").after(&["homebrew"]),
            "component(\"tools\")\ndef install(ctx):\n    pkg(\"brew\", \"git\")\n",
        );

    let (report, world) = run(
        &config,
        vec![ScriptedRun::ok("brew install git", "")],
        false,
        None,
    );
    assert!(report.succeeded(), "{report:?}");
    assert_eq!(world.exec.ran(), ["brew install git"]);
}

/// [R-PM-030] a package naming a manager nothing handles fails the component
/// that declared it, and says what is registered.
#[test]
fn a_package_with_no_handler_fails_the_component_that_declared_it() {
    let config = Config::new().declaring(
        Declaration::new("tools"),
        "component(\"tools\")\ndef install(ctx):\n    pkg(\"bwer\", \"git\")\n",
    );

    let (report, _) = run(&config, Vec::new(), false, None);
    let failure = report.failure.expect("it failed");
    assert_eq!(failure.component.as_str(), "tools");
    assert!(failure.reason.contains("bwer"), "{}", failure.reason);
}

/// [R-PM-031] a handler that raises fails the component that declared the
/// package, and the message names both files.
#[test]
fn a_handler_that_raises_names_both_components() {
    let handler = r#"
pm_name = "brew"
component("homebrew")
def install_pkg(ctx, name, version, **kwargs):
    fail("brew: command not found")
def uninstall_pkg(ctx, name, version, **kwargs):
    pass
def interrogate(ctx):
    return []
"#;
    let config = Config::new()
        .declaring(Declaration::new("homebrew"), handler)
        .declaring(
            Declaration::new("tools").after(&["homebrew"]),
            "component(\"tools\")\ndef install(ctx):\n    pkg(\"brew\", \"git\")\n",
        );

    let (report, _) = run(&config, Vec::new(), false, None);
    let failure = report.failure.expect("it failed");
    assert_eq!(failure.component.as_str(), "tools", "not the handler");
    assert!(failure.reason.contains("homebrew"), "{}", failure.reason);
    assert!(failure.reason.contains("tools"), "{}", failure.reason);
    assert!(failure.reason.contains("git"), "{}", failure.reason);
}

/// [R-PM-012] a handler with no `update_pkg` installs `latest` instead, which
/// is what most handlers rely on.
#[test]
fn updating_without_update_pkg_installs_latest() {
    let handler = r#"
pm_name = "brew"
component("homebrew")
def install_pkg(ctx, name, version, **kwargs):
    ctx.run("brew", ["install", name, version])
def uninstall_pkg(ctx, name, version, **kwargs):
    pass
def interrogate(ctx):
    return []
"#;
    let config = Config::new()
        .declaring(Declaration::new("homebrew"), handler)
        .declaring(
            Declaration::new("tools").after(&["homebrew"]),
            "component(\"tools\")\ndef update(ctx):\n    uppkg(\"brew\", \"git\")\n",
        );

    let loader = NoLoads;
    let platform = macos();
    let discovered = discover(&config, &loader, &platform).expect("discovery");
    let graph = Graph::build(&discovered).expect("the graph builds");
    let plan = Plan::compute(&graph, PhaseSet::Update, &Inputs::default());

    let (effects, world) = world(vec![ScriptedRun::ok("brew install git latest", "")], None);
    let evaluator = Evaluator::new(platform.clone(), &loader);
    let mut runner = Runner::new(
        &graph,
        &discovered.registry,
        evaluator,
        effects,
        settings(&platform, false),
    );
    let report = runner.run(&plan);

    assert!(report.succeeded(), "{report:?}");
    assert_eq!(world.exec.ran(), ["brew install git latest"]);
}

/// [R-ENGINE-051] the engine holds no renderer: everything a run says goes
/// out as an event. A test that reads only events sees the whole run.
#[test]
fn nothing_the_run_says_bypasses_the_event_stream() {
    let config = Config::new().declaring(
        Declaration::new("zsh"),
        "component(\"zsh\")\ndef install(ctx):\n    ctx.log(\"configuring\")\n",
    );
    let (_, world) = run(&config, Vec::new(), false, None);

    assert!(
        world
            .seen()
            .iter()
            .any(|e| matches!(e, Event::Message { text, .. } if text == "configuring")),
        "even a hook's own words are an event"
    );
}

/// [R-PM-020] `query_pm` reaches the handler for its manager and returns what
/// `interrogate` returned, which is a list of package names.
#[test]
fn query_pm_returns_what_the_handler_reports() {
    let handler = r#"
pm_name = "brew"
component("homebrew")
def install_pkg(ctx, name, version, **kwargs):
    pass
def uninstall_pkg(ctx, name, version, **kwargs):
    pass
def interrogate(ctx):
    result = ctx.run("brew", ["list"])
    return [line for line in result["stdout"].split("\n") if line]
"#;
    let config = Config::new()
        .declaring(Declaration::new("homebrew"), handler)
        .declaring(
            Declaration::new("tools").after(&["homebrew"]),
            "component(\"tools\")\ndef install(ctx):\n    for name in query_pm(\"brew\"):\n        ctx.write_file(\"~/\" + name, \"x\")\n",
        );

    let (report, world) = run(
        &config,
        vec![ScriptedRun::ok("brew list", "git\njq\n")],
        false,
        None,
    );
    assert!(report.succeeded(), "{report:?}");

    let written: Vec<String> = world
        .fs
        .paths()
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .filter(|n| n == "git" || n == "jq")
        .collect();
    assert_eq!(written, ["git", "jq"], "{:?}", world.fs.paths());
}

/// [R-PM-020] `interrogate` runs with the asking component's `ctx`, not the
/// handler's, so a handler reading `ctx.component_dir` sees the caller's.
#[test]
fn interrogate_runs_with_the_asking_components_ctx() {
    let handler = r#"
pm_name = "brew"
component("homebrew")
def install_pkg(ctx, name, version, **kwargs):
    pass
def uninstall_pkg(ctx, name, version, **kwargs):
    pass
def interrogate(ctx):
    return [ctx.component_dir]
"#;
    let config = Config::new()
        .declaring(Declaration::new("homebrew"), handler)
        .declaring(
            Declaration::new("tools").after(&["homebrew"]),
            "component(\"tools\")\ndef install(ctx):\n    for d in query_pm(\"brew\"):\n        ctx.log(d)\n",
        );

    let (report, world) = run(&config, Vec::new(), false, None);
    assert!(report.succeeded(), "{report:?}");
    assert!(
        world.seen().iter().any(|e| matches!(
            e,
            Event::Message { text, .. } if text.ends_with("tools")
        )),
        "the caller's component_dir, not the handler's: {:?}",
        world.seen()
    );
}

/// [R-PM-032] a handler returning the wrong shape is reported as the
/// handler's defect rather than coerced.
#[test]
fn an_interrogate_that_returns_the_wrong_shape_is_reported() {
    let handler = r#"
pm_name = "brew"
component("homebrew")
def install_pkg(ctx, name, version, **kwargs):
    pass
def uninstall_pkg(ctx, name, version, **kwargs):
    pass
def interrogate(ctx):
    return "git"
"#;
    let config = Config::new()
        .declaring(Declaration::new("homebrew"), handler)
        .declaring(
            Declaration::new("tools").after(&["homebrew"]),
            "component(\"tools\")\ndef install(ctx):\n    query_pm(\"brew\")\n",
        );

    let (report, _) = run(&config, Vec::new(), false, None);
    let failure = report.failure.expect("it failed");
    assert!(
        failure.reason.contains("list of strings"),
        "{}",
        failure.reason
    );
}

/// [R-PM-021] and [R-EXEC-011]: during a dry run whether a command runs
/// depends on the phase, and `interrogate` called from a read-only one still
/// gets its answer. A plan that could not ask what is installed would be
/// built on nothing.
#[test]
fn interrogate_runs_its_command_in_a_read_only_phase_of_a_dry_run() {
    let handler = r#"
pm_name = "brew"
component("homebrew")
def install_pkg(ctx, name, version, **kwargs):
    pass
def uninstall_pkg(ctx, name, version, **kwargs):
    pass
def interrogate(ctx):
    result = ctx.run("brew", ["list"])
    return [line for line in result["stdout"].split("\n") if line]
"#;
    let asking = |phase: &str| {
        format!(
            "component(\"tools\")\ndef {phase}(ctx):\n    ctx.log(str(len(query_pm(\"brew\"))))\n"
        )
    };

    for (phase, expected) in [("install_check", "1"), ("install", "0")] {
        let config = Config::new()
            .declaring(Declaration::new("homebrew"), handler)
            .declaring(
                Declaration::new("tools").after(&["homebrew"]),
                &asking(phase),
            );

        let loader = NoLoads;
        let platform = macos();
        let discovered = discover(&config, &loader, &platform).expect("discovery");
        let graph = Graph::build(&discovered).expect("the graph builds");
        let plan = Plan::compute(&graph, PhaseSet::Install, &Inputs::default());

        let (effects, world) = world(vec![ScriptedRun::ok("brew list", "git\n")], None);
        let evaluator = Evaluator::new(platform.clone(), &loader);
        let mut runner = Runner::new(
            &graph,
            &discovered.registry,
            evaluator,
            effects,
            meowctl_engine::Settings {
                dry_run: true,
                ..settings(&platform, false)
            },
        );
        let report = runner.run(&plan);

        assert!(report.succeeded(), "{phase}: {report:?}");
        assert!(
            world
                .seen()
                .iter()
                .any(|e| matches!(e, Event::Message { text, .. } if text == expected)),
            "{phase}: expected {expected}, saw {:?}",
            world.seen()
        );
    }
}
