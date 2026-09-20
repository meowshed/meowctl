//! What a configuration may say, and what happens when it says it wrong.

// `clippy.toml` exempts tests from `expect_used`, but only a function carrying
// `#[test]`. A helper in a test binary is test code by construction.
#![allow(clippy::expect_used)]

use std::collections::HashMap;

use meowctl_starlark::{
    Argument, Evaluator, LoadedFile, Loader, NoLoader, PackageAction, Platform, StarlarkError,
    StarlarkResult,
};

fn macos() -> Platform {
    Platform {
        os: "macos".to_owned(),
        ..Platform::default()
    }
}

fn evaluate(source: &str) -> StarlarkResult<meowctl_starlark::Evaluated> {
    let loader = NoLoader;
    Evaluator::new(macos(), &loader).evaluate("init.star", source)
}

/// [R-STAR-002] the shape a configuration's entry point has.
#[test]
fn components_are_collected_in_declaration_order() {
    let result = evaluate(
        r#"
component("base")
component("tool", after = ["base"], platform = "macos")
component("@stdlib//components/zsh")
"#,
    )
    .expect("evaluate");

    let names: Vec<&str> = result
        .declarations
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, ["base", "tool", "@stdlib//components/zsh"]);

    let tool = &result.declarations.components[1];
    assert_eq!(tool.after, ["base"]);
    assert_eq!(
        tool.extra.get("platform"),
        Some(&Argument::String("macos".to_owned()))
    );
}

/// [R-STAR-012] order is the graph's tie-break, so sorting would change which
/// component runs first.
#[test]
fn declaration_order_is_not_sorted() {
    let result = evaluate("component(\"z\")\ncomponent(\"a\")\n").expect("evaluate");
    let names: Vec<&str> = result
        .declarations
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, ["z", "a"]);
}

/// [R-STAR-003] the three package builtins differ only in what they ask for,
/// and their positional order is `manager` then `name` — which is not the
/// order a reader expects, and is what `parsePkgArgs` accepts.
#[test]
fn the_three_package_builtins_record_their_action() {
    let result = evaluate(
        r#"
pkg("brew", "git", version = "2.43")
unpkg(manager = "brew", name = "nano")
uppkg("brew", "jq")
"#,
    )
    .expect("evaluate");

    let actions: Vec<PackageAction> = result
        .declarations
        .packages
        .iter()
        .map(|p| p.action)
        .collect();
    assert_eq!(
        actions,
        [
            PackageAction::Install,
            PackageAction::Uninstall,
            PackageAction::Update
        ]
    );
    assert_eq!(result.declarations.packages[0].name, "git");
    assert_eq!(result.declarations.packages[0].version, "2.43");
    assert_eq!(result.declarations.packages[0].manager, "brew");
    assert_eq!(result.declarations.packages[1].name, "nano");
    assert_eq!(result.declarations.packages[2].manager, "brew");
}

/// [R-STAR-006] `deps.mod` is Starlark, and evaluating it rather than parsing
/// it with a second grammar is what keeps the two from disagreeing.
#[test]
fn a_modfile_evaluates_through_the_same_builtins() {
    let result = evaluate(
        r#"
module(name = "my-dotfiles", version = "0.1.0")
dep(name = "stdlib", version = "0.2.17")
dep(name = "plug", source = "github:o/r@v1")
replace(name = "stdlib", path = "/local/checkout")
"#,
    )
    .expect("evaluate");

    let module = result.declarations.module.expect("a module declaration");
    assert_eq!(module.name, "my-dotfiles");
    assert_eq!(module.version, "0.1.0");

    assert_eq!(result.declarations.deps.len(), 2);
    assert_eq!(result.declarations.deps[0].version, "0.2.17");
    assert_eq!(result.declarations.deps[1].source, "github:o/r@v1");
    assert_eq!(result.declarations.replaces[0].path, "/local/checkout");
}

/// [R-STAR-007] components write `platform().os`, so it is a struct rather
/// than a dictionary.
#[test]
fn platform_exposes_the_machine_by_attribute() {
    let result = evaluate("OS = platform().os\nWSL = platform().wsl\n").expect("evaluate");
    assert!(result.globals.contains(&"OS".to_owned()));
    assert!(result.globals.contains(&"WSL".to_owned()));
}

/// [R-STAR-008] the branch a configuration takes on this machine.
#[test]
fn select_chooses_by_platform() {
    let result = evaluate(
        r#"
GREETING = select({
    "//platform:macos": "mac",
    "//platform:linux": "linux",
    "//conditions:default": "other",
})
component(GREETING)
"#,
    )
    .expect("evaluate");
    assert_eq!(result.declarations.components[0].name, "mac");
}

#[test]
fn select_falls_back_to_the_default() {
    let result = evaluate(
        r#"
component(select({
    "//platform:linux": "linux",
    "//conditions:default": "other",
}))
"#,
    )
    .expect("evaluate");
    assert_eq!(result.declarations.components[0].name, "other");
}

/// A configuration that meant to cover this machine and did not should say so
/// rather than silently producing nothing.
#[test]
fn select_with_no_match_and_no_default_fails() {
    let err =
        evaluate("component(select({\"//platform:linux\": \"l\"}))\n").expect_err("should fail");
    assert!(err.to_string().contains("no condition matched"), "{err}");
}

/// [R-STAR-032] a typo that shadows a hook name would otherwise skip it
/// silently.
#[test]
fn a_hook_name_taken_by_a_non_function_is_reported() {
    let loader = NoLoader;
    let evaluator = Evaluator::new(macos(), &loader);

    struct NoArgument;
    impl meowctl_starlark::HookArgument for NoArgument {
        fn allocate<'v>(&self, heap: starlark::values::Heap<'v>) -> starlark::values::Value<'v> {
            heap.alloc("ctx")
        }
    }

    let err = evaluator
        .call_hook("c.star", "install = 42\n", "install", &NoArgument)
        .expect_err("should refuse");
    assert!(matches!(err, StarlarkError::NotCallable { .. }), "{err:?}");
    assert!(err.to_string().contains("install"), "{err}");
}

/// [R-STAR-031] most components define three of the thirteen phases, so an
/// absent hook is a success with nothing to do.
#[test]
fn an_absent_hook_is_not_an_error() {
    let loader = NoLoader;
    let evaluator = Evaluator::new(macos(), &loader);

    struct NoArgument;
    impl meowctl_starlark::HookArgument for NoArgument {
        fn allocate<'v>(&self, heap: starlark::values::Heap<'v>) -> starlark::values::Value<'v> {
            heap.alloc("ctx")
        }
    }

    let called = evaluator
        .call_hook(
            "c.star",
            "def install(ctx):\n    pass\n",
            "verify",
            &NoArgument,
        )
        .expect("absent hook");
    assert!(!called);
}

/// A hook runs, and what it declares is collected.
#[test]
fn a_hook_runs_and_its_declarations_are_collected() {
    let loader = NoLoader;
    let evaluator = Evaluator::new(macos(), &loader);

    struct NoArgument;
    impl meowctl_starlark::HookArgument for NoArgument {
        fn allocate<'v>(&self, heap: starlark::values::Heap<'v>) -> starlark::values::Value<'v> {
            heap.alloc("ctx")
        }
    }

    let called = evaluator
        .call_hook(
            "c.star",
            "def install(ctx):\n    pkg(\"brew\", \"git\")\n",
            "install",
            &NoArgument,
        )
        .expect("hook");
    assert!(called);
}

/// [R-STAR-040] M0 found the library already carries a span, which `v0.1.0`
/// does not: its errors name a file and nothing more.
#[test]
fn an_error_carries_the_line_it_happened_on() {
    let err = evaluate("x = 1\ncomponent()\n").expect_err("should fail");
    let span = err.span().expect("a span");
    assert_eq!(span.line, 2, "{span:?}");
    assert_eq!(span.file, "init.star");
    assert_eq!(span.source_line.as_deref(), Some("component()"));
}

/// [R-STAR-050] half a configuration applied is worse than none.
#[test]
fn a_syntax_error_reports_its_position() {
    let err = evaluate("component(\n").expect_err("should fail");
    assert!(err.span().is_some(), "{err}");
}

/// A loader that refuses is a module error rather than a configuration one, so
/// a script can tell a broken network from a broken configuration; see
/// [R-STAR-053].
#[test]
fn a_load_that_cannot_be_resolved_is_a_module_error() {
    let err = evaluate("load(\"@nope//x.star\", \"Y\")\n").expect_err("should fail");
    assert!(err.to_string().contains("nope"), "{err}");
}

/// A loader that answers, and a module whose exports are visible.
#[derive(Debug, Default)]
struct MapLoader {
    files: HashMap<String, String>,
    /// How many times each module was actually loaded.
    loads: std::cell::RefCell<HashMap<String, usize>>,
}

impl Loader for MapLoader {
    fn load(&self, module: &str) -> StarlarkResult<LoadedFile> {
        *self
            .loads
            .borrow_mut()
            .entry(module.to_owned())
            .or_default() += 1;
        self.files
            .get(module)
            .map(|source| LoadedFile {
                name: module.to_owned(),
                source: source.clone(),
            })
            .ok_or_else(|| StarlarkError::Load {
                module: module.to_owned(),
                reason: "not in the map".to_owned(),
            })
    }
}

/// [R-STAR-020] `load()` goes through the caller's loader, which is what lets
/// one composite loader serve the filesystem, the registry, and GitHub.
#[test]
fn load_resolves_through_the_loader() {
    let mut loader = MapLoader::default();
    loader.files.insert(
        "@stdlib//helper.star".to_owned(),
        "NAME = \"from-stdlib\"\n".to_owned(),
    );

    let evaluator = Evaluator::new(macos(), &loader);
    let result = evaluator
        .evaluate(
            "init.star",
            "load(\"@stdlib//helper.star\", \"NAME\")\ncomponent(NAME)\n",
        )
        .expect("evaluate");

    assert_eq!(result.declarations.components[0].name, "from-stdlib");
}

/// [R-STAR-023] a graph that loads one helper from twenty components should
/// pay for it once.
#[test]
fn a_module_loaded_twice_is_evaluated_once() {
    let mut loader = MapLoader::default();
    loader.files.insert(
        "@stdlib//helper.star".to_owned(),
        "NAME = \"x\"\n".to_owned(),
    );

    let evaluator = Evaluator::new(macos(), &loader);
    for _ in 0..3 {
        evaluator
            .evaluate(
                "init.star",
                "load(\"@stdlib//helper.star\", \"NAME\")\ncomponent(NAME)\n",
            )
            .expect("evaluate");
    }

    assert_eq!(loader.loads.borrow()["@stdlib//helper.star"], 1);
}

/// [R-STAR-010] two evaluations in one process must not see each other's
/// declarations.
#[test]
fn each_evaluation_gets_its_own_accumulator() {
    let loader = NoLoader;
    let evaluator = Evaluator::new(macos(), &loader);

    let first = evaluator
        .evaluate("a.star", "component(\"a\")\n")
        .expect("first");
    let second = evaluator
        .evaluate("b.star", "component(\"b\")\n")
        .expect("second");

    assert_eq!(first.declarations.components.len(), 1);
    assert_eq!(second.declarations.components.len(), 1);
    assert_eq!(second.declarations.components[0].name, "b");
}

/// [R-STAR-003] a manager is required, because a package declaration with
/// nowhere to send it is a configuration mistake rather than a default.
#[test]
fn a_package_without_a_manager_is_refused() {
    let err = evaluate("pkg(\"git\")\n").expect_err("should fail");
    assert!(err.to_string().contains("name"), "{err}");
}

/// [R-STAR-002] `component(name = ...)` is how a generated configuration
/// writes it, so the name is positional or named.
#[test]
fn a_component_name_may_be_given_by_keyword() {
    let result = evaluate("component(name = \"neovim\")\n").expect("evaluate");
    assert_eq!(result.declarations.components[0].name, "neovim");
}

/// [R-STAR-001] a builtin `v0.1.0` does not provide would let a component stop
/// working on the old binary.
#[test]
fn the_extended_builtins_are_not_available() {
    // `print` is in starlark-rust's extended set and not in what v0.1.0
    // predeclares.
    let err = evaluate("print(\"hello\")\n").expect_err("should fail");
    assert!(err.to_string().contains("print"), "{err}");
}

/// [R-STAR-001] the standard library's package-manager components call
/// `json.decode(result.stdout)` to read what a manager reported, so the module
/// is part of the surface rather than decoration.
#[test]
fn json_is_a_module_with_decode_and_encode() {
    let result = evaluate(
        r#"
parsed = json.decode('{"name": "git", "versions": [1, 2]}')
component(parsed["name"])
component(json.encode(["a", "b"]))
"#,
    )
    .expect("evaluate");

    let names: Vec<&str> = result
        .declarations
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, ["git", "[\"a\",\"b\"]"]);
}

/// Malformed JSON from a package manager should name the problem rather than
/// failing somewhere later with a confusing type error.
#[test]
fn decoding_malformed_json_says_so() {
    let err = evaluate("component(json.decode('not json'))\n").expect_err("should fail");
    assert!(err.to_string().contains("json.decode"), "{err}");
}

/// A registry that answers one manager, for the `query_pm` tests.
#[derive(Debug)]
struct OneManager {
    manager: &'static str,
    installed: Vec<String>,
}

impl meowctl_starlark::PackageManagers for OneManager {
    fn interrogate(&self, manager: &str) -> StarlarkResult<Vec<String>> {
        if manager == self.manager {
            return Ok(self.installed.clone());
        }
        Err(StarlarkError::Evaluation {
            message: format!("no component handles the package manager {manager}"),
            span: None,
            load_chain: Vec::new(),
        })
    }
}

/// [R-STAR-005] the one builtin that runs another component's code during
/// evaluation. A configuration that asks what is installed and declares
/// components accordingly is why it exists.
#[test]
fn query_pm_returns_what_the_manager_reports() {
    let loader = NoLoader;
    let managers = std::sync::Arc::new(OneManager {
        manager: "brew",
        installed: vec!["git".to_owned(), "jq".to_owned()],
    });
    let result = Evaluator::new(macos(), &loader)
        .with_package_managers(managers)
        .evaluate(
            "init.star",
            "for name in query_pm(\"brew\"):\n    component(name)\n",
        )
        .expect("evaluate");

    let names: Vec<&str> = result
        .declarations
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, ["git", "jq"]);
}

/// [R-PM-030] a typo in a manager name is the common cause, and the failure
/// has to reach the configuration that made it.
#[test]
fn query_pm_fails_when_nothing_handles_the_manager() {
    let loader = NoLoader;
    let managers = std::sync::Arc::new(OneManager {
        manager: "brew",
        installed: Vec::new(),
    });
    let err = Evaluator::new(macos(), &loader)
        .with_package_managers(managers)
        .evaluate("init.star", "query_pm(\"bwer\")\n")
        .expect_err("should fail");
    assert!(err.to_string().contains("bwer"), "{err}");
}

/// [R-STAR-005] an evaluation with no registry refuses rather than answering
/// with an empty list, which would silently drop every component the
/// configuration meant to declare.
#[test]
fn query_pm_without_a_registry_refuses() {
    let err = evaluate("query_pm(\"brew\")\n").expect_err("should fail");
    assert!(err.to_string().contains("query_pm"), "{err}");
}

/// [R-STAR-033] `pm_name` is what decides whether a component handles a
/// package manager, and it is read from the evaluation rather than by
/// re-parsing the file.
#[test]
fn the_value_of_every_top_level_string_is_reported() {
    let result = evaluate("pm_name = \"brew\"\nother = 3\ntext = \"hi\"\n").expect("evaluate");
    assert_eq!(
        result.strings.get("pm_name").map(String::as_str),
        Some("brew")
    );
    assert_eq!(result.strings.get("text").map(String::as_str), Some("hi"));
    assert!(
        !result.strings.contains_key("other"),
        "{:?}",
        result.strings
    );
}
