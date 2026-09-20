//! What a configuration declares, and the order it runs in.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

mod support;

use std::collections::BTreeMap;

use meowctl_common::ComponentId;
use meowctl_engine::{Declaration, EngineError, discover};
use support::{Config, NoLoads, graph_of, linux, macos, plain};

/// [R-ENGINE-013] declaration order is the tie-break, so two runs of the same
/// configuration produce the same order and a plan is worth reading.
#[test]
fn components_with_no_edge_between_them_keep_declaration_order() {
    let config = Config::new()
        .declaring(Declaration::new("zsh"), &plain("zsh"))
        .declaring(Declaration::new("neovim"), &plain("neovim"))
        .declaring(Declaration::new("git"), &plain("git"));

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    assert_eq!(graph.names(), ["zsh", "neovim", "git"]);
}

/// [R-ENGINE-013] and an edge wins over declaration order.
#[test]
fn a_component_runs_after_what_it_names() {
    let config = Config::new()
        .declaring(
            Declaration::new("neovim").after(&["mise"]),
            &plain("neovim"),
        )
        .declaring(Declaration::new("mise"), &plain("mise"));

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    assert_eq!(graph.names(), ["mise", "neovim"]);
}

/// [R-ENGINE-014] a component file may name what it runs after itself, which
/// is what keeps a component from being a change to `init.star` as well.
#[test]
fn a_component_file_may_name_its_own_dependencies() {
    let config = Config::new()
        .declaring(
            Declaration::new("neovim"),
            "after = [\"mise\"]\ncomponent(\"neovim\")\n",
        )
        .declaring(Declaration::new("mise"), &plain("mise"));

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    assert_eq!(graph.names(), ["mise", "neovim"]);
}

/// [R-ENGINE-014] and a name the configuration does not declare is pulled in
/// rather than ignored: a component that names a dependency is relying on it.
#[test]
fn an_undeclared_dependency_is_pulled_into_the_graph() {
    let config = Config::new()
        .declaring(
            Declaration::new("test-mise").after(&["@stdlib//components/mise"]),
            &plain("test-mise"),
        )
        .holding(
            "@stdlib//components/mise",
            &plain("@stdlib//components/mise"),
        );

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    assert_eq!(graph.names(), ["mise", "test-mise"]);
}

/// [R-ENGINE-011] and what it pulls in is marked, because a `meowctl remove`
/// of one tool must not run the uninstall hook of the package manager it was
/// reached through.
#[test]
fn a_component_reached_through_a_dependency_is_not_declared() {
    let config = Config::new()
        .declaring(
            Declaration::new("test-mise").after(&["mise"]),
            &plain("test-mise"),
        )
        .holding("mise", &plain("mise"));

    let loader = NoLoads;
    let discovered = discover(&config, &loader, &macos()).expect("discovery");
    let declared: BTreeMap<&str, bool> = discovered
        .components
        .iter()
        .map(|c| (c.logical_name(), c.declared))
        .collect();
    assert_eq!(declared.get("test-mise"), Some(&true));
    assert_eq!(declared.get("mise"), Some(&false));
}

/// [R-ENGINE-011] transitively, so declaring an aggregate brings in what it
/// needs without copying its whole tree into `init.star`.
#[test]
fn dependencies_are_followed_all_the_way_down() {
    let config = Config::new()
        .declaring(Declaration::new("top").after(&["middle"]), &plain("top"))
        .holding("middle", "after = [\"bottom\"]\ncomponent(\"middle\")\n")
        .holding("bottom", &plain("bottom"));

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    assert_eq!(graph.names(), ["bottom", "middle", "top"]);
}

/// [R-ENGINE-010] a component declared in both `init.star` and `local.star`
/// counts once, and the first declaration is the one that stands.
#[test]
fn a_component_declared_twice_counts_once() {
    let config = Config::new()
        .declaring(Declaration::new("zsh").after(&["git"]), &plain("zsh"))
        .declaring(Declaration::new("git"), &plain("git"));

    let mut config = config;
    config.declared.push(Declaration::new("zsh"));

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    assert_eq!(
        graph.names(),
        ["git", "zsh"],
        "the first declaration stands"
    );
}

/// [R-ENGINE-012] a component declaring a `platforms` list that does not hold
/// this machine's operating system is dropped.
#[test]
fn a_platform_guard_drops_a_component() {
    let config = Config::new()
        .declaring(Declaration::new("zsh"), &plain("zsh"))
        .declaring(
            Declaration::new("apt"),
            "platforms = [\"linux\"]\ncomponent(\"apt\")\n",
        );

    assert_eq!(
        graph_of(&config, &macos()).expect("on macOS").names(),
        ["zsh"]
    );
    assert_eq!(
        graph_of(&config, &linux("debian", ""))
            .expect("on Linux")
            .names(),
        ["zsh", "apt"]
    );
}

/// [R-ENGINE-012] a `distros` guard matches the distribution or its `ID_LIKE`
/// by equality. `select()` matches `ID_LIKE` by substring; these are not the
/// same rule, whatever an earlier draft of the requirement said.
#[test]
fn a_distro_guard_matches_the_distribution_or_its_id_like() {
    let config = Config::new().declaring(
        Declaration::new("apt"),
        "distros = [\"debian\"]\ncomponent(\"apt\")\n",
    );

    assert_eq!(
        graph_of(&config, &linux("debian", ""))
            .expect("on debian")
            .names(),
        ["apt"]
    );
    assert_eq!(
        graph_of(&config, &linux("linuxmint", "debian"))
            .expect("on mint")
            .names(),
        ["apt"],
        "ID_LIKE is the other half of the match"
    );
    assert!(
        graph_of(&config, &linux("arch", ""))
            .expect("on arch")
            .names()
            .is_empty()
    );
}

/// [R-ENGINE-012] a component declaring neither guard runs everywhere, and a
/// guard that is not a list of strings is ignored rather than refused.
#[test]
fn a_component_with_no_usable_guard_runs_everywhere() {
    let config = Config::new()
        .declaring(Declaration::new("zsh"), &plain("zsh"))
        .declaring(
            Declaration::new("odd"),
            "platforms = \"linux\"\ncomponent(\"odd\")\n",
        );

    assert_eq!(
        graph_of(&config, &macos()).expect("builds").names(),
        ["zsh", "odd"]
    );
}

/// [R-ENGINE-052] a component a guard dropped is reported rather than being
/// silently absent, because "it did not run" is not something a user can act
/// on.
#[test]
fn a_dropped_component_says_which_guard_dropped_it() {
    let config = Config::new().declaring(
        Declaration::new("apt"),
        "platforms = [\"linux\"]\ncomponent(\"apt\")\n",
    );

    let loader = NoLoads;
    let discovered = discover(&config, &loader, &macos()).expect("discovery");
    assert_eq!(discovered.excluded.len(), 1);
    let (id, guard) = &discovered.excluded[0];
    assert_eq!(id.as_str(), "apt");
    assert!(guard.contains("linux"), "{guard}");
}

/// [R-ENGINE-015] a cycle names the components on it. `TopoSort` reports only
/// that there is one, which leaves a user with a hundred components and no
/// way to find the two that point at each other.
#[test]
fn a_cycle_names_the_components_on_it() {
    let config = Config::new()
        .declaring(Declaration::new("a").after(&["b"]), &plain("a"))
        .declaring(Declaration::new("b").after(&["a"]), &plain("b"))
        .declaring(Declaration::new("elsewhere"), &plain("elsewhere"));

    let err = graph_of(&config, &macos()).expect_err("the cycle is reported");
    let EngineError::Cycle { on_it } = &err else {
        panic!("expected a cycle, got {err}");
    };
    let names: Vec<&str> = on_it
        .iter()
        .map(meowctl_common::ComponentId::as_str)
        .collect();
    assert_eq!(names, ["a", "b"], "and nothing else: {err}");
}

/// [R-ENGINE-016] naming a component restricts the run to it and what it
/// needs. Excluding the dependency is what made `meowctl apply test-mise`
/// fail on a tool that was never installed.
#[test]
fn a_filter_keeps_what_the_named_components_depend_on() {
    let config = Config::new()
        .declaring(Declaration::new("mise"), &plain("mise"))
        .declaring(
            Declaration::new("test-mise").after(&["mise"]),
            &plain("test-mise"),
        )
        .declaring(Declaration::new("unrelated"), &plain("unrelated"));

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    let scoped = graph
        .restricted_to(&["test-mise".to_owned()])
        .expect("the filter matches");
    assert_eq!(scoped.names(), ["mise", "test-mise"]);
}

/// [R-ENGINE-016] a name matching nothing is an error, because an empty run
/// reports success.
#[test]
fn a_filter_matching_nothing_is_an_error() {
    let config = Config::new().declaring(Declaration::new("zsh"), &plain("zsh"));
    let graph = graph_of(&config, &macos()).expect("the graph builds");

    let err = graph
        .restricted_to(&["nonesuch".to_owned()])
        .expect_err("should refuse");
    assert!(err.to_string().contains("nonesuch"), "{err}");
}

/// [R-ENGINE-020] and [R-PM-003]: every component is evaluated once before any
/// hook runs, so a hook in the first can declare a package the last handles.
#[test]
fn package_manager_handlers_are_registered_during_discovery() {
    let handler = r#"
pm_name = "brew"
component("homebrew")
def install_pkg(ctx, name, version, **kwargs):
    pass
def uninstall_pkg(ctx, name, version, **kwargs):
    pass
def interrogate(ctx):
    return []
"#;
    let config = Config::new()
        .declaring(
            Declaration::new("developer-tools"),
            &plain("developer-tools"),
        )
        .declaring(Declaration::new("homebrew"), handler);

    let loader = NoLoads;
    let discovered = discover(&config, &loader, &macos()).expect("discovery");
    assert_eq!(discovered.registry.managers(), ["brew"]);
}

/// [R-PM-004] two components claiming one manager is reported, where `v0.1.0`
/// lets whichever evaluated last win.
#[test]
fn two_components_claiming_one_manager_fail_discovery() {
    let handler = |name: &str| {
        format!(
            r#"
pm_name = "brew"
component({name:?})
def install_pkg(ctx, name, version, **kwargs):
    pass
def uninstall_pkg(ctx, name, version, **kwargs):
    pass
def interrogate(ctx):
    return []
"#
        )
    };
    let config = Config::new()
        .declaring(Declaration::new("homebrew"), &handler("homebrew"))
        .declaring(Declaration::new("linuxbrew"), &handler("linuxbrew"));

    let loader = NoLoads;
    let err = discover(&config, &loader, &macos()).expect_err("should refuse");
    assert!(err.to_string().contains("brew"), "{err}");
}

/// [R-ENGINE-021] the second pass reuses what the first found rather than
/// evaluating the file again, which is what makes the two passes one read.
#[test]
fn what_a_component_exports_is_kept_for_the_second_pass() {
    let config = Config::new().declaring(
        Declaration::new("zsh"),
        "component(\"zsh\")\ndef install(ctx):\n    pass\n",
    );

    let loader = NoLoads;
    let discovered = discover(&config, &loader, &macos()).expect("discovery");
    assert!(discovered.components[0].evaluated.has_hook("install"));
}

/// A name that is not a component identifier fails where it is written rather
/// than becoming a registry lookup for something nonsensical.
#[test]
fn a_declaration_that_is_not_a_component_is_refused() {
    let config = Config::new().declaring(Declaration::new(""), "");
    let loader = NoLoads;
    let err = discover(&config, &loader, &macos()).expect_err("should refuse");
    assert!(matches!(err, EngineError::UnusableName { .. }), "{err}");
}

/// A component file that does not evaluate names the file, because a
/// configuration with a hundred components needs to say which one.
#[test]
fn a_component_that_does_not_evaluate_names_itself() {
    let config = Config::new().declaring(Declaration::new("broken"), "component(\n");
    let loader = NoLoads;
    let err = discover(&config, &loader, &macos()).expect_err("should refuse");
    assert!(err.to_string().contains("broken"), "{err}");
}

/// [R-ENGINE-018] a module's components refer to each other by name. Reading
/// `after = ["fish-config"]` inside `@dotmeow` against the configuration
/// looks for a file the user never wrote, and fails on every configuration
/// that uses an aggregate module.
#[test]
fn a_bare_dependency_inside_a_module_resolves_inside_that_module() {
    let config = Config::new()
        .declaring(
            Declaration::new("@dotmeow"),
            "after = [\"fish-config\"]\ncomponent(\"@dotmeow\")\n",
        )
        .holding(
            "@dotmeow//components/fish-config",
            &plain("@dotmeow//components/fish-config"),
        );

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    // `@dotmeow` keeps its `@`: `logicalName` strips a path, not a sigil.
    assert_eq!(graph.names(), ["fish-config", "@dotmeow"]);
}

/// [R-ENGINE-018] and the rule is about where the name is written, not about
/// the name: the same bare name in the configuration's own component means a
/// component in the configuration.
#[test]
fn a_bare_dependency_in_the_configuration_resolves_in_the_configuration() {
    let config = Config::new()
        .declaring(Declaration::new("mine").after(&["helper"]), &plain("mine"))
        .holding("helper", &plain("helper"));

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    assert_eq!(graph.names(), ["helper", "mine"]);
}

/// [R-ENGINE-018] a name that is already qualified is left alone, whoever
/// wrote it: `@dotmeow` depending on `@stdlib//components/tmux` means that
/// one, not a `tmux` of its own.
#[test]
fn a_qualified_dependency_is_left_alone_inside_a_module() {
    let config = Config::new()
        .declaring(
            Declaration::new("@dotmeow").after(&["@stdlib//components/tmux"]),
            &plain("@dotmeow"),
        )
        .holding(
            "@stdlib//components/tmux",
            &plain("@stdlib//components/tmux"),
        );

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    assert_eq!(graph.names(), ["tmux", "@dotmeow"]);
}

/// [R-ENGINE-018] a name already carrying `@` is left alone, and so is one
/// carrying `//`. Only a bare name is read against its declarer's module.
///
/// The two are separate cases: `@dotmeow` has the sigil and no path, and
/// `github.com/o/r//x` has the path and no sigil. Requiring both would
/// prefix the first into `@dotmeow//components/@dotmeow`.
#[test]
fn a_name_that_is_already_qualified_is_left_as_written() {
    let declarer: ComponentId = "@dotmeow//components/fish".parse().expect("an id");

    for already in [
        "@dotmeow",
        "@stdlib//components/git",
        "github.com/o/r//components/x",
    ] {
        assert_eq!(
            meowctl_engine::resolve_bare(&declarer, already),
            already,
            "{already} was rewritten"
        );
    }

    // And a bare name is the one that moves.
    assert_eq!(
        meowctl_engine::resolve_bare(&declarer, "fish-config"),
        "@dotmeow//components/fish-config"
    );
}

/// [R-ENGINE-018] a component that belongs to no module leaves a bare name
/// alone, because there is no module for it to mean.
#[test]
fn a_bare_name_declared_outside_a_module_stays_bare() {
    let local: ComponentId = "zsh".parse().expect("an id");
    assert_eq!(
        meowctl_engine::resolve_bare(&local, "fish-config"),
        "fish-config"
    );
}

/// [R-ENGINE-013] an edge is followed once every component it waits on is
/// done, and not before. A component waiting on two is not freed by one.
#[test]
fn a_component_waiting_on_two_is_not_freed_by_one() {
    let config = Config::new()
        .declaring(Declaration::new("first"), "component(\"first\")\n")
        .declaring(Declaration::new("second"), "component(\"second\")\n")
        .declaring(
            Declaration::new("last").after(&["first", "second"]),
            "component(\"last\")\n",
        );

    let graph = graph_of(&config, &macos()).expect("the graph builds");
    let order: Vec<&str> = graph.names();
    let last = order.iter().position(|n| *n == "last").expect("last is in");
    let first = order
        .iter()
        .position(|n| *n == "first")
        .expect("first is in");
    let second = order
        .iter()
        .position(|n| *n == "second")
        .expect("second is in");

    assert!(last > first, "{order:?}");
    assert!(last > second, "{order:?}");
}
