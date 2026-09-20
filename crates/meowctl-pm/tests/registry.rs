//! Which component handles which manager, and what gets called on it.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::collections::BTreeMap;

use meowctl_pm::{Handler, HandlerFailure, PmError, Registration, Registry, scan};
use meowctl_starlark::{
    Argument, Evaluator, NoLoader, PackageAction, PackageDecl, Platform, RepoDecl,
};

/// A handler component with the exports named, and nothing else.
fn component(pm_name: Option<&str>, exports: &[&str]) -> meowctl_starlark::Evaluated {
    let mut source = String::new();
    if let Some(name) = pm_name {
        source.push_str(&format!("pm_name = \"{name}\"\n"));
    }
    for export in exports {
        source.push_str(&format!("def {export}(ctx, *args, **kwargs):\n    pass\n"));
    }
    let loader = NoLoader;
    Evaluator::new(Platform::default(), &loader)
        .evaluate("handler.star", &source)
        .expect("the handler evaluates")
}

/// The four exports [R-PM-001] requires, and nothing optional.
fn minimal() -> meowctl_starlark::Evaluated {
    component(
        Some("brew"),
        &["install_pkg", "uninstall_pkg", "interrogate"],
    )
}

fn handler(manager: &str, component: &str) -> Handler {
    Handler {
        manager: manager.to_owned(),
        component: component.to_owned(),
        has_update: false,
        has_add_repo: false,
    }
}

fn install(manager: &str, name: &str, version: &str) -> PackageDecl {
    PackageDecl {
        action: PackageAction::Install,
        name: name.to_owned(),
        version: version.to_owned(),
        manager: manager.to_owned(),
        extra: BTreeMap::new(),
    }
}

/// [R-PM-001] all four, or it is not a handler.
#[test]
fn a_component_exporting_all_four_is_a_handler() {
    let registration = scan("homebrew", &minimal());
    assert_eq!(
        registration,
        Registration::Handler(Handler {
            manager: "brew".to_owned(),
            component: "homebrew".to_owned(),
            has_update: false,
            has_add_repo: false,
        })
    );
}

/// [R-PM-001] a component that names a manager and exports half of what a
/// handler needs is nearly always a mistake, so it is reported rather than
/// ignored.
#[test]
fn a_component_missing_a_required_export_is_reported_by_name() {
    let evaluated = component(Some("brew"), &["install_pkg"]);
    let Registration::Incomplete { manager, missing } = scan("homebrew", &evaluated) else {
        panic!("expected an incomplete registration");
    };
    assert_eq!(manager, "brew");
    assert_eq!(missing, ["uninstall_pkg", "interrogate"]);
}

#[test]
fn a_component_that_names_no_manager_is_not_a_handler() {
    let evaluated = component(None, &["install_pkg"]);
    assert_eq!(scan("neovim", &evaluated), Registration::NotAHandler);
}

/// [R-PM-002] the two optional exports are what a handler may or may not have,
/// and each changes what dispatch does.
#[test]
fn the_optional_exports_are_recorded() {
    let evaluated = component(
        Some("brew"),
        &[
            "install_pkg",
            "uninstall_pkg",
            "interrogate",
            "update_pkg",
            "add_repo",
        ],
    );
    let Registration::Handler(handler) = scan("homebrew", &evaluated) else {
        panic!("expected a handler");
    };
    assert!(handler.has_update);
    assert!(handler.has_add_repo);
}

/// [R-PM-004] `v0.1.0` overwrites the earlier handler, so which one wins
/// depends on the order components happened to evaluate in.
#[test]
fn two_components_claiming_one_manager_are_reported() {
    let mut registry = Registry::new();
    registry
        .register(handler("brew", "homebrew"))
        .expect("the first");
    let err = registry
        .register(handler("brew", "linuxbrew"))
        .expect_err("the second");
    assert_eq!(
        err,
        PmError::DuplicateHandler {
            manager: "brew".to_owned(),
            first: "homebrew".to_owned(),
            second: "linuxbrew".to_owned(),
        }
    );
}

/// Registering the same component again is an evaluation that ran twice, not a
/// conflict: two components is the thing worth stopping.
#[test]
fn registering_the_same_component_twice_is_not_a_conflict() {
    let mut registry = Registry::new();
    registry
        .register(handler("brew", "homebrew"))
        .expect("the first");
    registry
        .register(handler("brew", "homebrew"))
        .expect("again");
    assert_eq!(registry.managers(), ["brew"]);
}

/// [R-PM-010] what a `pkg()` turns into.
#[test]
fn a_package_declaration_becomes_a_call_to_install_pkg() {
    let mut registry = Registry::new();
    registry
        .register(handler("brew", "homebrew"))
        .expect("register");

    let mut declaration = install("brew", "git", "2.43.0");
    declaration
        .extra
        .insert("cask".to_owned(), Argument::Boolean(true));

    let call = registry.call_for(&declaration).expect("dispatch");
    assert_eq!(call.component, "homebrew");
    assert_eq!(call.function, "install_pkg");
    assert_eq!(call.positional, ["git", "2.43.0"]);
    assert_eq!(call.keyword.get("cask"), Some(&Argument::Boolean(true)));
}

/// [R-PM-011] the other two actions go to their own functions.
#[test]
fn removing_and_updating_go_to_their_own_functions() {
    let mut registry = Registry::new();
    registry
        .register(Handler {
            has_update: true,
            ..handler("brew", "homebrew")
        })
        .expect("register");

    let remove = PackageDecl {
        action: PackageAction::Uninstall,
        ..install("brew", "git", "")
    };
    assert_eq!(
        registry.call_for(&remove).expect("dispatch").function,
        "uninstall_pkg"
    );

    let update = PackageDecl {
        action: PackageAction::Update,
        ..install("brew", "git", "")
    };
    let call = registry.call_for(&update).expect("dispatch");
    assert_eq!(call.function, "update_pkg");
    assert_eq!(call.positional, ["git"], "update_pkg takes no version");
}

/// [R-PM-012] removing the fallback would break every handler that never
/// defined an update path, which is most of them.
#[test]
fn updating_without_update_pkg_installs_latest_instead() {
    let mut registry = Registry::new();
    registry
        .register(handler("brew", "homebrew"))
        .expect("register");

    let update = PackageDecl {
        action: PackageAction::Update,
        ..install("brew", "git", "2.43.0")
    };
    let call = registry.call_for(&update).expect("dispatch");
    assert_eq!(call.function, "install_pkg");
    assert_eq!(
        call.positional,
        ["git", "latest"],
        "the declared version is replaced, not kept"
    );
}

/// [R-PM-013] a `repo()` on a handler with no `add_repo` has nowhere to go,
/// and saying so beats doing nothing.
#[test]
fn a_repository_declaration_needs_add_repo() {
    let mut registry = Registry::new();
    registry
        .register(Handler {
            has_add_repo: true,
            ..handler("apt", "apt")
        })
        .expect("register apt");
    registry
        .register(handler("brew", "homebrew"))
        .expect("register brew");

    let declaration = RepoDecl {
        manager: "apt".to_owned(),
        arguments: BTreeMap::from([(
            "url".to_owned(),
            Argument::String("https://example.invalid".to_owned()),
        )]),
    };
    let call = registry.call_for_repo(&declaration).expect("dispatch");
    assert_eq!(call.function, "add_repo");
    assert!(call.positional.is_empty(), "add_repo takes only ctx");

    let refused = RepoDecl {
        manager: "brew".to_owned(),
        ..declaration
    };
    let err = registry.call_for_repo(&refused).expect_err("no add_repo");
    assert_eq!(
        err,
        PmError::NoAddRepo {
            manager: "brew".to_owned(),
            component: "homebrew".to_owned(),
        }
    );
}

/// [R-PM-030] a typo in a manager name is the common cause, and the list of
/// what is registered is what makes it obvious.
#[test]
fn an_unhandled_manager_lists_the_ones_that_are_handled() {
    let mut registry = Registry::new();
    registry
        .register(handler("brew", "homebrew"))
        .expect("register");
    registry.register(handler("apt", "apt")).expect("register");

    let err = registry
        .call_for(&install("bwer", "git", ""))
        .expect_err("no handler");
    let message = err.to_string();
    assert!(message.contains("bwer"), "{message}");
    assert!(message.contains("apt, brew"), "{message}");
}

/// The message has to be usable before anything is registered too, which is
/// the state a configuration with no package-manager component is in.
#[test]
fn an_empty_registry_says_so_rather_than_listing_nothing() {
    let registry = Registry::new();
    let err = registry
        .call_for(&install("brew", "git", ""))
        .expect_err("no handler");
    assert!(err.to_string().contains("none"), "{err}");
}

/// [R-PM-020] interrogation is a call like any other, made on the handler's
/// component.
#[test]
fn interrogation_is_a_call_to_the_handlers_component() {
    let mut registry = Registry::new();
    registry
        .register(handler("brew", "homebrew"))
        .expect("register");
    let call = registry.call_for_interrogate("brew").expect("dispatch");
    assert_eq!(call.component, "homebrew");
    assert_eq!(call.function, "interrogate");
    assert!(call.positional.is_empty());
    assert!(call.keyword.is_empty());
}

/// [R-PM-014] every call names the handler's component, and the caller's `ctx`
/// is what the engine passes: the handler's effects belong to the component
/// that asked for the package.
#[test]
fn a_call_names_the_component_to_evaluate_and_nothing_about_the_caller() {
    let mut registry = Registry::new();
    registry
        .register(handler("brew", "homebrew"))
        .expect("register");
    let call = registry
        .call_for(&install("brew", "git", ""))
        .expect("dispatch");
    assert_eq!(call.component, "homebrew");
    assert_eq!(call.manager, "brew");
}

/// [R-PM-031] a handler that raises must fail the component that declared the
/// package. A message naming only the handler sends the reader to a file they
/// did not write.
#[test]
fn a_handler_failure_names_both_components() {
    let err = PmError::from(HandlerFailure {
        manager: "brew".to_owned(),
        package: "git".to_owned(),
        component: "developer-tools".to_owned(),
        handler: "homebrew".to_owned(),
        function: "install_pkg".to_owned(),
        reason: "brew: command not found".to_owned(),
    });
    let message = err.to_string();
    assert!(message.contains("developer-tools"), "{message}");
    assert!(message.contains("homebrew"), "{message}");
    assert!(message.contains("git"), "{message}");
    assert!(message.contains("brew: command not found"), "{message}");
}

/// [R-PM-032] a handler returning the wrong shape is a defect in the handler,
/// and coercing it would hide which of the two files is wrong.
#[test]
fn a_handler_returning_the_wrong_shape_is_reported_as_the_handlers_defect() {
    let err = PmError::HandlerReturned {
        handler: "homebrew".to_owned(),
        function: "interrogate".to_owned(),
        found: "string".to_owned(),
        expected: "a list of strings".to_owned(),
    };
    let message = err.to_string();
    assert!(message.contains("homebrew"), "{message}");
    assert!(message.contains("interrogate"), "{message}");
    assert!(message.contains("a list of strings"), "{message}");
}
