//! What a dry run leaves behind, which is nothing.
//!
//! Driven through the binary because what [R-OPS-026] constrains is the whole
//! run: the journal is never opened, so there is nothing inside `meowctl-ops`
//! to ask. A journal of things that did not happen would replay into damage.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::PathBuf;
use std::process::Command;

/// A configuration whose component writes a file, so a real run would have
/// something to journal.
fn sandbox() -> PathBuf {
    let root = std::env::temp_dir().join(format!("meowctl-dryrun-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("components")).expect("the sandbox");
    std::fs::write(root.join("init.star"), "component(\"writer\")\n").expect("init.star");
    std::fs::write(
        root.join("components").join("writer.star"),
        "def install(ctx):\n    ctx.write_file(ctx.home + \"/written\", \"x\")\n",
    )
    .expect("the component");
    root
}

/// [R-OPS-026] no journal, and no journal file.
#[test]
fn a_dry_run_opens_no_journal() {
    let root = sandbox();
    let output = Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args([
            "--config",
            &root.display().to_string(),
            "apply",
            "--dry-run",
            "--force",
        ])
        .output()
        .expect("the binary runs");
    assert!(output.status.success(), "{output:?}");

    assert!(
        !root.join("rollback.jsonl").exists(),
        "a dry run opened a journal"
    );
}

/// [R-CLI-011] and [R-FS-012]: a dry run records nothing either, so the next
/// real run does the work rather than skipping it as already done.
#[test]
fn a_dry_run_records_no_run() {
    let root = sandbox();
    Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args([
            "--config",
            &root.display().to_string(),
            "apply",
            "--dry-run",
        ])
        .output()
        .expect("the binary runs");

    assert!(
        !root.join("state.toml").exists(),
        "a dry run wrote the sentinel"
    );
}
