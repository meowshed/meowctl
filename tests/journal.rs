//! What a run leaves in the journal, driven through the binary.
//!
//! Driven through the binary because [REQ-2025] constrains the whole run: the
//! journal belongs to `apply`, and nothing inside `meowctl-ops` decides when a
//! run has succeeded. A journal a successful run leaves behind is replayed by
//! the next failed one, which undoes work nobody asked to undo (BUG-0001).

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A configuration whose one component writes `~/a.txt`, with `HOME` inside
/// the sandbox so nothing outside it is touched.
fn sandbox(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("meowctl-journal-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("config").join("components")).expect("the sandbox");
    std::fs::create_dir_all(root.join("home")).expect("the home");
    std::fs::write(root.join("config").join("init.star"), "component(\"a\")\n").expect("init.star");
    std::fs::write(
        root.join("config").join("components").join("a.star"),
        "def install(ctx):\n    ctx.write_file(\"~/a.txt\", \"a\")\n",
    )
    .expect("component a");
    root
}

fn apply(root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .env("HOME", root.join("home"))
        .args([
            "--config",
            &root.join("config").display().to_string(),
            "apply",
            "--format",
            "plain",
        ])
        .output()
        .expect("the binary runs")
}

/// [REQ-2025] a run that succeeded leaves nothing for a later run to undo.
#[test]
fn a_successful_run_leaves_an_empty_journal() {
    let root = sandbox("success");
    let output = apply(&root);
    assert!(output.status.success(), "{output:?}");

    let journal = root.join("config").join("rollback.jsonl");
    let left = std::fs::read_to_string(&journal).unwrap_or_default();
    assert!(left.trim().is_empty(), "the journal still holds: {left}");
}

/// [REQ-2025] BUG-0001's reproduction: a later run that fails undoes its own
/// work and leaves the file an earlier successful run wrote.
#[test]
fn a_failed_run_does_not_undo_an_earlier_successful_one() {
    let root = sandbox("failure");
    assert!(apply(&root).status.success(), "the first run");

    let config = root.join("config");
    std::fs::write(
        config.join("components").join("b.star"),
        "def install(ctx):\n    ctx.write_file(\"~/b.txt\", \"b\")\n    fail(\"boom\")\n",
    )
    .expect("component b");
    std::fs::write(
        config.join("init.star"),
        "component(\"a\")\ncomponent(\"b\")\n",
    )
    .expect("init.star");
    let output = apply(&root);
    assert!(!output.status.success(), "b fails: {output:?}");

    assert!(
        root.join("home").join("a.txt").exists(),
        "the failed run undid the earlier one: {output:?}"
    );
    assert!(
        !root.join("home").join("b.txt").exists(),
        "the failed run kept its own write"
    );
}
