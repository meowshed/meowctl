//! Where a component's file is looked for.
//!
//! Driven through the binary because the lookup is `ConfigSources` in
//! `meowctl-cli`: the engine asks for a component by name and the CLI decides
//! which of the two spellings on disk answers; see [R-ENGINE-017].

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

fn sandbox(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("meowctl-layout-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("components")).expect("the sandbox");
    root
}

fn apply(root: &Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args([
            "--config",
            &root.display().to_string(),
            "apply",
            "--dry-run",
            "--force",
        ])
        .output()
        .expect("the binary runs")
}

/// [R-ENGINE-017] a hand-written configuration keeps its components as files.
#[test]
fn a_bare_name_resolves_to_a_file_beside_the_others() {
    let root = sandbox("flat");
    std::fs::write(root.join("init.star"), "component(\"zsh\")\n").expect("init.star");
    std::fs::write(
        root.join("components").join("zsh.star"),
        "def install(ctx):\n    pass\n",
    )
    .expect("the component");

    let output = apply(&root);
    assert!(output.status.success(), "{output:?}");
}

/// [R-ENGINE-017] and the standard library keeps them as directories, because
/// a component with data files needs somewhere to put them.
#[test]
fn a_bare_name_resolves_to_a_directory_of_its_own() {
    let root = sandbox("nested");
    std::fs::write(root.join("init.star"), "component(\"nvim\")\n").expect("init.star");
    let dir = root.join("components").join("nvim");
    std::fs::create_dir_all(&dir).expect("the directory");
    std::fs::write(dir.join("init.star"), "def install(ctx):\n    pass\n").expect("the component");

    let output = apply(&root);
    assert!(output.status.success(), "{output:?}");
}

/// [R-ENGINE-017] a name that is neither names both spellings, because a
/// reader who wrote one of them needs to know which was looked for.
#[test]
fn a_name_that_is_neither_names_both_spellings() {
    let root = sandbox("absent");
    std::fs::write(root.join("init.star"), "component(\"ghost\")\n").expect("init.star");

    let output = apply(&root);
    let said = String::from_utf8_lossy(&output.stderr);
    assert!(said.contains("components/ghost.star"), "{said}");
    assert!(said.contains("components/ghost/init.star"), "{said}");
}
