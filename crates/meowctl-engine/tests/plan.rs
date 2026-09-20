//! What a run will do, before it does any of it.
//!
//! The plan is the one thing a dry run prints and a real run executes, so
//! every test here is really about the same question: does the plan say what
//! the runner will do? `v0.1.0` printed every component in the graph and the
//! runner skipped most of them, which is the defect these pin down.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

mod support;

use meowctl_common::{Phase, PhaseSet, SkipReason};
use meowctl_config::{CompletedComponent, Sentinel};
use meowctl_engine::{Declaration, Inputs, Plan};
use support::{Config, graph_of, macos, plain};

/// A sentinel recording these phase-and-component pairs as done.
fn sentinel(done: &[(&str, &str)]) -> Sentinel {
    Sentinel {
        completed_components: done
            .iter()
            .map(|(phase, component)| CompletedComponent {
                phase: (*phase).to_owned(),
                component: (*component).to_owned(),
                completed_at: None,
            })
            .collect(),
        ..Sentinel::default()
    }
}

fn two_components() -> Config {
    Config::new()
        .declaring(Declaration::new("zsh"), &plain("zsh"))
        .declaring(Declaration::new("neovim"), &plain("neovim"))
}

/// [R-ENGINE-002] the plan names the phases, the components in each, and the
/// order. The order is the set's order of phases and the graph's order of
/// components within each.
#[test]
fn the_plan_is_every_phase_of_the_set_over_every_component() {
    let graph = graph_of(&two_components(), &macos()).expect("the graph builds");
    let plan = Plan::compute(&graph, PhaseSet::Install, &Inputs::default());

    let steps: Vec<(Phase, &str)> = plan
        .steps
        .iter()
        .map(|s| (s.phase, s.component.as_str()))
        .collect();
    assert_eq!(
        steps,
        [
            (Phase::InstallCheck, "zsh"),
            (Phase::InstallCheck, "neovim"),
            (Phase::Install, "zsh"),
            (Phase::Install, "neovim"),
            (Phase::InstallConfigure, "zsh"),
            (Phase::InstallConfigure, "neovim"),
        ]
    );
}

/// [R-ENGINE-002] a component whose phase is already recorded is reported as
/// skipped rather than listed as pending work. Listing it is what made the
/// dry run say 120 components before and after a successful apply alike.
#[test]
fn a_completed_phase_is_skipped_and_says_so() {
    let graph = graph_of(&two_components(), &macos()).expect("the graph builds");
    let done = sentinel(&[("install", "zsh"), ("install_check", "zsh")]);
    let plan = Plan::compute(
        &graph,
        PhaseSet::Install,
        &Inputs {
            sentinel: Some(&done),
            ..Inputs::default()
        },
    );

    let skipped: Vec<(Phase, &str)> = plan
        .steps
        .iter()
        .filter(|s| s.skipped == Some(SkipReason::AlreadyCompleted))
        .map(|s| (s.phase, s.component.as_str()))
        .collect();
    assert_eq!(
        skipped,
        [(Phase::InstallCheck, "zsh"), (Phase::Install, "zsh")],
        "the third phase is not recorded, so it still runs"
    );
    assert_eq!(plan.running_in(Phase::Install), 1);
}

/// `--force` re-runs everything, which is what `Runner.Force` means and what
/// a user reaching for it is asking for.
#[test]
fn force_runs_everything_regardless_of_what_completed() {
    let graph = graph_of(&two_components(), &macos()).expect("the graph builds");
    let done = sentinel(&[("install", "zsh"), ("install", "neovim")]);
    let plan = Plan::compute(
        &graph,
        PhaseSet::Install,
        &Inputs {
            sentinel: Some(&done),
            force: true,
            ..Inputs::default()
        },
    );
    assert!(plan.steps.iter().all(|s| s.skipped.is_none()), "{plan:?}");
}

/// [R-ENGINE-052] a component the filter leaves out is reported as filtered
/// out rather than being absent, because a component a user expected and
/// cannot find is a bug report.
#[test]
fn a_filtered_component_is_named_with_its_reason() {
    let config = two_components().declaring(Declaration::new("git"), &plain("git"));
    let graph = graph_of(&config, &macos()).expect("the graph builds");
    let filter = ["zsh".to_owned()];
    let plan = Plan::compute(
        &graph,
        PhaseSet::Install,
        &Inputs {
            filter: &filter,
            ..Inputs::default()
        },
    );

    let install: Vec<(&str, Option<&SkipReason>)> = plan
        .steps
        .iter()
        .filter(|s| s.phase == Phase::Install)
        .map(|s| (s.component.as_str(), s.skipped.as_ref()))
        .collect();
    assert_eq!(
        install,
        [
            ("zsh", None),
            ("neovim", Some(&SkipReason::FilteredOut)),
            ("git", Some(&SkipReason::FilteredOut)),
        ]
    );
}

/// [R-ENGINE-016] and a filter keeps what the named components depend on, so
/// the plan does not promise to install a tool without its package manager.
#[test]
fn a_filter_keeps_the_dependencies_of_what_it_names() {
    let config = Config::new()
        .declaring(Declaration::new("mise"), &plain("mise"))
        .declaring(
            Declaration::new("test-mise").after(&["mise"]),
            &plain("test-mise"),
        );
    let graph = graph_of(&config, &macos()).expect("the graph builds");
    let filter = ["test-mise".to_owned()];
    let plan = Plan::compute(
        &graph,
        PhaseSet::Install,
        &Inputs {
            filter: &filter,
            ..Inputs::default()
        },
    );

    let running: Vec<&str> = plan
        .running()
        .filter(|s| s.phase == Phase::Install)
        .map(|s| s.component.as_str())
        .collect();
    assert_eq!(running, ["mise", "test-mise"]);
}

/// [R-ENGINE-052] a component a guard dropped is in the plan with the guard,
/// rather than missing from it.
#[test]
fn a_component_a_guard_dropped_is_in_the_plan_with_its_reason() {
    let config = two_components().declaring(
        Declaration::new("apt"),
        "platforms = [\"linux\"]\ncomponent(\"apt\")\n",
    );
    let loader = support::NoLoads;
    let discovered = meowctl_engine::discover(&config, &loader, &macos()).expect("discovery");
    let graph = meowctl_engine::Graph::build(&discovered).expect("the graph builds");
    let plan = Plan::compute(
        &graph,
        PhaseSet::Install,
        &Inputs {
            excluded: &discovered.excluded,
            ..Inputs::default()
        },
    );

    let apt = plan
        .steps
        .iter()
        .find(|s| s.component.as_str() == "apt" && s.phase == Phase::Install)
        .expect("apt is in the plan");
    let Some(SkipReason::PlatformMismatch { guard }) = &apt.skipped else {
        panic!("expected a platform mismatch, got {:?}", apt.skipped);
    };
    assert!(guard.contains("linux"), "{guard}");
}

/// [R-ENGINE-003] a plan with nothing to do says so, which is what lets a
/// command report "nothing to do" instead of a phase heading and no lines.
#[test]
fn a_plan_with_nothing_to_run_is_empty() {
    let graph = graph_of(&two_components(), &macos()).expect("the graph builds");
    let done = sentinel(&[
        ("install_check", "zsh"),
        ("install", "zsh"),
        ("install_configure", "zsh"),
        ("install_check", "neovim"),
        ("install", "neovim"),
        ("install_configure", "neovim"),
    ]);
    let plan = Plan::compute(
        &graph,
        PhaseSet::Install,
        &Inputs {
            sentinel: Some(&done),
            ..Inputs::default()
        },
    );
    assert!(plan.is_empty());
    assert_eq!(plan.steps.len(), 6, "and says what it skipped: {plan:?}");
}

/// The other phase sets run their own phases and nothing else.
#[test]
fn each_phase_set_plans_its_own_phases() {
    let graph = graph_of(&two_components(), &macos()).expect("the graph builds");
    for (set, expected) in [
        (PhaseSet::Update, vec![Phase::Update]),
        (PhaseSet::Verify, vec![Phase::Verify]),
        (
            PhaseSet::Uninstall,
            vec![
                Phase::UninstallCheck,
                Phase::Uninstall,
                Phase::UninstallCleanup,
            ],
        ),
    ] {
        let plan = Plan::compute(&graph, set, &Inputs::default());
        let phases: Vec<Phase> = plan
            .steps
            .iter()
            .map(|s| s.phase)
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect();
        assert_eq!(phases, expected, "{set}");
    }
}
