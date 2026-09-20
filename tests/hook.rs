//! `meowctl hook`, run as the binary a shell runs.
//!
//! The only test here that drives the process rather than a library, and it
//! has to: what [R-CLI-062] promises is about the exit code and the streams,
//! which a function call cannot observe. A shell evaluates this stdout, so
//! the assertions are on exact bytes.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

/// A configuration directory holding one component with the given source.
fn sandbox(name: &str, component: &str, source: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("meowctl-hook-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("components")).expect("the sandbox");
    std::fs::write(
        root.join("init.star"),
        format!("component({component:?})\n"),
    )
    .expect("init.star");
    std::fs::write(
        root.join("components").join(format!("{component}.star")),
        source,
    )
    .expect("the component");
    root
}

fn hook(root: &Path, phase: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args(["--config", &root.display().to_string(), "hook", phase])
        .env("SHELL", "/bin/zsh")
        .output()
        .expect("the binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// [R-CLI-061] what the hooks emitted, and nothing around it. A glyph or an
/// indent here would be evaluated as a command.
#[test]
fn what_the_hooks_emitted_is_the_whole_of_stdout() {
    let root = sandbox(
        "emits",
        "shellbits",
        "def shell(ctx):\n    ctx.emit(\"export A=1\")\n    ctx.emit(\"export B=2\")\n",
    );

    let output = hook(&root, "shell");
    assert!(output.status.success(), "{output:?}");
    assert_eq!(stdout(&output), "export A=1\nexport B=2\n");
}

/// [R-CTX-002] a component chooses between `set -gx` and `export` by reading
/// `ctx.shell`, so the runtime hook has to name it.
#[test]
fn the_hook_knows_which_shell_it_is_contributing_to() {
    let root = sandbox(
        "shellname",
        "shellbits",
        "def shell(ctx):\n    ctx.emit(\"shell is \" + ctx.shell)\n",
    );

    assert_eq!(stdout(&hook(&root, "shell")), "shell is zsh\n");
}

/// [R-CLI-062] a shell that cannot start is worse than a shell that starts
/// without its integration, so the failure is recorded and the command still
/// succeeds.
#[test]
fn a_failing_hook_is_recorded_and_exits_zero() {
    let root = sandbox(
        "failing",
        "broken",
        "def shell(ctx):\n    fail(\"deliberate\")\n",
    );

    let output = hook(&root, "shell");
    assert!(output.status.success(), "the exit code stays 0");
    assert_eq!(stdout(&output), "", "nothing reaches the shell");

    let flag = std::fs::read_to_string(root.join(".hook-error")).expect("the flag");
    let mut lines = flag.lines();
    let at = lines.next().expect("a timestamp");
    assert!(at.ends_with('Z') && at.contains('T'), "{at} is RFC 3339");
    assert!(flag.contains("deliberate"), "{flag}");
}

/// [R-CLI-063] a run in which nothing failed removes it, so a fixed
/// configuration stops warning.
#[test]
fn a_clean_run_clears_the_flag() {
    let root = sandbox(
        "clearing",
        "broken",
        "def shell(ctx):\n    fail(\"deliberate\")\n",
    );
    hook(&root, "shell");
    assert!(root.join(".hook-error").exists());

    std::fs::write(
        root.join("components").join("broken.star"),
        "def shell(ctx):\n    ctx.emit(\"export OK=1\")\n",
    )
    .expect("the fix");

    assert_eq!(stdout(&hook(&root, "shell")), "export OK=1\n");
    assert!(!root.join(".hook-error").exists(), "the flag was cleared");
}

/// [R-CLI-060] only the two phases a shell spawn runs. The rest belong to a
/// phase set and are reached through `apply`.
#[test]
fn a_phase_that_is_not_a_runtime_hook_is_a_usage_error() {
    let root = sandbox("usage", "shellbits", "def shell(ctx):\n    pass\n");

    for phase in ["install", "verify", "nonesuch"] {
        let output = hook(&root, phase);
        assert_eq!(output.status.code(), Some(2), "{phase}: {output:?}");
        let said = String::from_utf8_lossy(&output.stderr);
        assert!(said.contains("shell") && said.contains("login"), "{said}");
    }
}

/// [R-CLI-060] and both of the two are accepted.
#[test]
fn both_runtime_hook_phases_are_accepted() {
    let root = sandbox(
        "bothphases",
        "shellbits",
        "def shell(ctx):\n    ctx.emit(\"s\")\n\ndef login(ctx):\n    ctx.emit(\"l\")\n",
    );

    assert_eq!(stdout(&hook(&root, "shell")), "s\n");
    assert_eq!(stdout(&hook(&root, "login")), "l\n");
}

fn report(root: &Path, command: &str) -> Output {
    Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args(["--config", &root.display().to_string(), command])
        .output()
        .expect("the binary runs")
}

/// [R-CLI-064] the user meets the failure here, because `hook` itself says
/// nothing. `status` says that it happened and `doctor` says what it was.
#[test]
fn status_and_doctor_report_the_flag() {
    let root = sandbox(
        "reporting",
        "broken",
        "def shell(ctx):\n    fail(\"deliberate\")\n",
    );
    hook(&root, "shell");

    let said = stdout(&report(&root, "status"));
    assert!(said.contains("runtime hook"), "{said}");
    assert!(said.contains("doctor"), "status points at doctor: {said}");

    let diagnosed = stdout(&report(&root, "doctor"));
    assert!(diagnosed.contains("deliberate"), "{diagnosed}");
}

/// [R-CLI-064] and neither says anything when there is nothing to say.
#[test]
fn nothing_is_reported_when_the_last_run_was_clean() {
    let root = sandbox(
        "quiet",
        "shellbits",
        "def shell(ctx):\n    ctx.emit(\"x\")\n",
    );
    hook(&root, "shell");

    assert!(!stdout(&report(&root, "status")).contains("runtime hook"));
    assert!(!stdout(&report(&root, "doctor")).contains("runtime hook"));
}

/// [R-CLI-065] nothing a shell spawn does is journalled, so a rollback on the
/// next failure cannot undo the previous spawn.
#[test]
fn a_runtime_hook_opens_no_journal() {
    let root = sandbox(
        "journal",
        "shellbits",
        "def shell(ctx):\n    ctx.emit(\"export A=1\")\n",
    );
    hook(&root, "shell");

    assert!(
        !root.join("rollback.jsonl").exists(),
        "a shell spawn left a journal behind"
    );
}
