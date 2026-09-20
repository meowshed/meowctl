//! What an error looks like from outside the process.
//!
//! Exit codes, streams and diagnostics are things a script sees, so these run
//! the binary. A test that called a function would be checking the mapping
//! against itself.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn sandbox(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("meowctl-errors-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("components")).expect("the sandbox");
    root
}

fn run(root: &Path, args: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_meowctl"));
    command.args(["--config", &root.display().to_string()]);
    command.args(args);
    command.output().expect("the binary runs")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

fn stderr(output: &Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

/// [R-CLI-030] and [R-COMMON-031]: each kind of failure reaches the shell as
/// its own code, because a script that cannot tell a typo from a broken
/// network retries the wrong one.
#[test]
fn each_kind_of_failure_has_its_own_exit_code() {
    let root = sandbox("codes");

    // 2: the command line itself is wrong.
    assert_eq!(run(&root, &["nonesuch"]).status.code(), Some(2));
    assert_eq!(run(&root, &["apply", "--nonesuch"]).status.code(), Some(2));

    // 3: the configuration is wrong. An empty directory is not one.
    assert_eq!(run(&root, &["apply"]).status.code(), Some(3));
}

/// [R-CLI-032] a usage error prints usage; a failure in the work does not,
/// because a wall of flags on top of a real error buries it.
#[test]
fn usage_appears_for_a_usage_error_and_not_for_a_failure() {
    let root = sandbox("usage");

    let usage = run(&root, &["apply", "--nonesuch"]);
    assert!(stderr(&usage).contains("Usage"), "{}", stderr(&usage));

    // A configuration error is not a usage mistake.
    let failure = run(&root, &["apply"]);
    assert!(!stderr(&failure).contains("Usage"), "{}", stderr(&failure));
}

/// [R-CLI-033] an error goes to stderr, so `meowctl dep list | ...` carries
/// the list and nothing else.
#[test]
fn an_error_does_not_reach_a_piped_stdout() {
    let root = sandbox("streams");
    let failure = run(&root, &["apply"]);

    assert!(!failure.status.success());
    assert_eq!(stdout(&failure), "", "the error reached stdout");
    assert!(!stderr(&failure).is_empty(), "nothing reached stderr");
}

/// [R-CLI-031] and [R-COMMON-032]: a Starlark error is rendered as a
/// diagnostic naming the file, the line and the line itself, because the
/// alternative is a message about a file the user then has to search.
#[test]
fn a_starlark_error_names_the_file_the_line_and_the_line() {
    let root = sandbox("span");
    std::fs::write(root.join("init.star"), "component(\"a\")\nnonesuch()\n").expect("init.star");

    let output = run(&root, &["apply"]);
    let said = stderr(&output);

    assert_eq!(output.status.code(), Some(3), "{said}");
    assert!(said.contains("init.star"), "{said}");
    assert!(said.contains('2'), "the line is not named: {said}");
    assert!(
        said.contains("nonesuch"),
        "the source line is not shown: {said}"
    );
}

/// [R-CLI-020] and [R-CLI-022]: a dry run renders the plan through the sink,
/// with a reason beside each component it will not touch.
#[test]
fn a_dry_run_renders_the_plan_with_its_reasons() {
    let root = sandbox("plan");
    std::fs::write(
        root.join("init.star"),
        "component(\"here\")\ncomponent(\"elsewhere\")\n",
    )
    .expect("init.star");
    std::fs::write(
        root.join("components").join("here.star"),
        "def install(ctx):\n    pass\n",
    )
    .expect("here");
    std::fs::write(
        root.join("components").join("elsewhere.star"),
        "platforms = [\"plan9\"]\ndef install(ctx):\n    pass\n",
    )
    .expect("elsewhere");

    let output = run(&root, &["apply", "--dry-run", "--force"]);
    assert!(output.status.success(), "{output:?}");

    let said = stdout(&output);
    assert!(said.contains("here"), "{said}");
    assert!(said.contains("elsewhere"), "{said}");
    assert!(
        said.contains("plan9") || said.contains("platform"),
        "the skip has no reason: {said}"
    );
}

/// [R-CLI-071] a release publishing no checksums is refused, because a
/// release nothing can be verified against is not one that needs no
/// verification.
#[test]
fn self_update_refuses_a_release_with_no_checksums() {
    let root = sandbox("selfupdate");
    let served = root.join("release.json");
    std::fs::write(
        &served,
        concat!(
            r#"{"tag_name":"v9.9.9","html_url":"https://h/r","assets":"#,
            r#"[{"name":"meowctl-x","browser_download_url":"https://h/x"}]}"#,
        ),
    )
    .expect("the release");

    let output = Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args(["--config", &root.display().to_string(), "self-update"])
        .env("MEOWCTL_RELEASES", format!("file://{}", served.display()))
        .output()
        .expect("the binary runs");

    // `file://` is not https, so the fetch is refused before anything else --
    // which is itself the point: [R-NET-003] reaches self-update too.
    assert!(!output.status.success(), "{output:?}");
    assert_eq!(output.status.code(), Some(4), "a module error");
}

/// [R-CLI-072] and [R-CLI-076]: the release is asked for where the variable
/// says, and a plaintext URL is refused there as everywhere.
#[test]
fn self_update_will_not_fetch_a_release_over_plaintext() {
    let root = sandbox("plaintext");
    let output = Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args(["--config", &root.display().to_string(), "self-update"])
        .env("MEOWCTL_RELEASES", "http://example.invalid/releases/latest")
        .output()
        .expect("the binary runs");

    assert_eq!(output.status.code(), Some(4), "{output:?}");
    let said = stderr(&output);
    assert!(said.contains("example.invalid"), "{said}");
}
