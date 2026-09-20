//! Changing a file that belongs to the user.
//!
//! Every test here asserts on the whole file rather than on the line that
//! changed, because what matters is the part that did not.

// `clippy.toml` exempts tests from `expect_used`, but only a function carrying
// `#[test]`. A helper in a test binary is test code by construction.
#![allow(clippy::expect_used)]

use meowctl_config::edit;

/// A configuration with the things a user puts in one: a header, section
/// comments, blank lines, and a declaration written by keyword.
const CONFIG: &str = r#"# init.star — my dotfiles
#
# Work machine.

# --- shared ---

component("@dotmeow")

# --- editors ---

component("neovim")
component(name = "helix")

# --- opt in per machine ---
# component("kubernetes")
"#;

/// [R-CONFIG-050] the file belongs to the user. This is the test the whole
/// design decision was for.
#[test]
fn adding_a_component_changes_nothing_else() {
    let after = edit::add_component("init.star", CONFIG, "ripgrep").expect("add");

    assert_eq!(
        after,
        format!("{CONFIG}component(\"ripgrep\")\n"),
        "something other than the addition moved"
    );
    assert!(after.contains("# --- editors ---"));
    assert!(after.contains("# component(\"kubernetes\")"));
}

/// [R-CONFIG-052] `AppendComponent` in `v0.1.0` appends whatever it is given,
/// so running `meowctl add` twice leaves two declarations.
#[test]
fn adding_a_component_that_is_already_there_does_nothing() {
    let after = edit::add_component("init.star", CONFIG, "neovim").expect("add");
    assert_eq!(after, CONFIG);
}

/// A commented-out declaration is a comment. Text matching would find it.
#[test]
fn a_commented_out_declaration_does_not_count_as_declared() {
    assert!(
        !edit::has_component("init.star", CONFIG, "kubernetes").expect("check"),
        "a comment was read as a declaration"
    );

    let after = edit::add_component("init.star", CONFIG, "kubernetes").expect("add");
    assert!(after.ends_with("component(\"kubernetes\")\n"), "{after}");
    // The comment is still a comment.
    assert!(after.contains("# component(\"kubernetes\")"));
}

/// A name inside a string is not a declaration either.
#[test]
fn a_name_inside_a_string_does_not_count_as_declared() {
    let source = "MESSAGE = \"component(\\\"ghost\\\")\"\n";
    assert!(!edit::has_component("init.star", source, "ghost").expect("check"));
}

/// [R-CONFIG-050] removing takes the declaration and the newline that ended
/// it, and leaves the comment above it alone: a section header belongs to
/// whoever wrote it, not to the entry below.
#[test]
fn removing_a_component_leaves_the_comments_around_it() {
    let after = edit::remove_component("init.star", CONFIG, "neovim").expect("remove");

    assert!(!after.contains("component(\"neovim\")"), "{after}");
    assert!(after.contains("# --- editors ---"), "{after}");
    assert!(after.contains("component(name = \"helix\")"), "{after}");
    assert!(
        !after.contains("\n\n\n"),
        "a blank line was left where the declaration was:\n{after}"
    );
}

/// A declaration written by keyword is the same declaration.
#[test]
fn a_component_named_by_keyword_is_found() {
    assert!(edit::has_component("init.star", CONFIG, "helix").expect("check"));
    let after = edit::remove_component("init.star", CONFIG, "helix").expect("remove");
    assert!(!after.contains("helix"), "{after}");
}

/// [R-CONFIG-052] a silent no-op leaves the user believing something happened.
#[test]
fn removing_a_component_that_is_not_there_is_reported() {
    let err = edit::remove_component("init.star", CONFIG, "absent").expect_err("should report");
    assert!(err.to_string().contains("absent"), "{err}");
}

const MANIFEST: &str = r#"# meowctl.mod — machine-managed module file. Do not edit by hand.

module(
    name = "my-dotfiles",
    version = "0.1.0",
)

dep(name = "stdlib", version = "0.2.17")
dep(name = "plug", source = "github:o/r@v1")
"#;

/// [R-CONFIG-051] the ordinary case, which `v0.1.0` also handles.
#[test]
fn a_dep_version_changes_and_the_rest_of_the_file_does_not() {
    let after = edit::set_dep_version("deps.mod", MANIFEST, "stdlib", "0.3.0").expect("bump");

    assert_eq!(
        after,
        MANIFEST.replace("version = \"0.2.17\"", "version = \"0.3.0\"")
    );
}

/// [R-CONFIG-051] the case `v0.1.0` documents as unsupported: its regular
/// expression requires `name` before `version`, so a hand-edited manifest
/// reports "not found" for a declaration that is plainly there.
#[test]
fn a_dep_written_in_the_other_keyword_order_is_still_found() {
    let manifest = "dep(version = \"0.2.17\", name = \"stdlib\")\n";
    let after = edit::set_dep_version("deps.mod", manifest, "stdlib", "0.3.0").expect("bump");

    assert!(after.contains("0.3.0"), "{after}");
    assert!(after.contains("stdlib"), "{after}");
}

/// A `dep()` carrying a source keeps it: the replacement is built from what
/// the declaration said, not from a template.
#[test]
fn a_dep_with_a_source_keeps_it() {
    let after = edit::set_dep_version("deps.mod", MANIFEST, "plug", "2.0.0").expect("bump");
    assert!(after.contains("source = \"github:o/r@v1\""), "{after}");
    assert!(after.contains("version = \"2.0.0\""), "{after}");
}

/// [R-CONFIG-052] again, for the manifest.
#[test]
fn bumping_a_module_that_is_not_declared_is_reported() {
    let err =
        edit::set_dep_version("deps.mod", MANIFEST, "absent", "1.0.0").expect_err("should report");
    assert!(err.to_string().contains("absent"), "{err}");
}

/// [R-CONFIG-053] `meowctl add` after removing the last component has to have
/// somewhere to write.
#[test]
fn removing_the_last_component_leaves_a_file_that_still_parses() {
    let source = "component(\"only\")\n";
    let after = edit::remove_component("init.star", source, "only").expect("remove");

    assert_eq!(after, "");
    let back = edit::add_component("init.star", &after, "next").expect("add");
    assert_eq!(back, "component(\"next\")\n");
}

/// A file that does not parse is reported rather than edited into something
/// worse.
#[test]
fn an_unparseable_file_is_refused() {
    let err = edit::add_component("init.star", "component(\n", "x").expect_err("should refuse");
    assert!(err.to_string().contains("init.star"), "{err}");
}

/// [R-CONFIG-010] the dialect is named in two crates, so a file this module
/// accepts has to be one the evaluator accepts. A disagreement would mean
/// `meowctl add` succeeding on a configuration that then fails to apply.
#[test]
fn the_editor_and_the_evaluator_accept_the_same_file() {
    use meowctl_starlark::{Evaluator, NoLoader, Platform};

    let edited = edit::add_component("init.star", CONFIG, "ripgrep").expect("add");

    let loader = NoLoader;
    let evaluator = Evaluator::new(Platform::current(), &loader);
    let result = evaluator
        .evaluate("init.star", &edited)
        .expect("the edited file should evaluate");

    let names: Vec<&str> = result
        .declarations
        .components
        .iter()
        .map(|c| c.name.as_str())
        .collect();
    assert_eq!(names, ["@dotmeow", "neovim", "helix", "ripgrep"]);
}
