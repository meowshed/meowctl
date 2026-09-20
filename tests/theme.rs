//! The theme file, read by the binary before it draws anything.
//!
//! Driven through the binary because what [R-TUI-056] promises is about when
//! the file is read and what happens when it cannot be: the sink is chosen
//! once, so a fallback has to happen before the first line.

// `clippy.toml` exempts a function carrying `#[test]`; a helper in a test
// binary is test code by construction.
#![allow(clippy::expect_used)]

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn sandbox(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("meowctl-theme-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("components")).expect("the sandbox");
    std::fs::write(root.join("init.star"), "").expect("init.star");
    root
}

/// `status` because it needs a configuration directory and touches nothing.
fn status(root: &Path) -> Output {
    Command::new(env!("CARGO_BIN_EXE_meowctl"))
        .args(["--config", &root.display().to_string(), "status"])
        .env("CLICOLOR_FORCE", "1")
        .env("TERM", "xterm-256color")
        .env("COLORTERM", "truecolor")
        .output()
        .expect("the binary runs")
}

fn said(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

/// [R-TUI-056] no file is the ordinary case and says nothing. A warning on
/// every command for a file the user never wrote is noise.
#[test]
fn no_theme_file_is_silent() {
    let root = sandbox("absent");
    let output = status(&root);

    assert!(output.status.success(), "{output:?}");
    assert!(!said(&output).contains("theme"), "{}", said(&output));
}

/// [R-TUI-050] a theme the user wrote reaches what is drawn.
#[test]
fn a_theme_file_changes_what_is_drawn() {
    let root = sandbox("applied");
    std::fs::write(
        root.join("theme.toml"),
        "[info]\nr = 255\ng = 0\nb = 0\nansi16 = 31\n",
    )
    .expect("the theme");

    let output = status(&root);
    assert!(output.status.success(), "{output:?}");
    assert!(
        said(&output).contains("255;0;0"),
        "the palette did not reach the line: {:?}",
        said(&output)
    );
}

/// [R-TUI-052] a malformed theme warns and falls back. Nobody's apply stops
/// because their colours are wrong.
#[test]
fn a_malformed_theme_warns_and_the_command_still_runs() {
    let root = sandbox("malformed");
    std::fs::write(root.join("theme.toml"), "[accent]\nr = \"not a number\"\n").expect("the theme");

    let output = status(&root);
    assert!(
        output.status.success(),
        "a bad theme failed the command: {output:?}"
    );

    let text = said(&output);
    assert!(text.contains("theme.toml"), "{text}");
    assert!(text.contains("default"), "{text}");
}

/// [R-TUI-052] and a misspelled role is the same kind of mistake, because a
/// table that silently does nothing is worse than one that complains.
#[test]
fn a_misspelled_role_warns_rather_than_being_ignored() {
    let root = sandbox("misspelled");
    std::fs::write(
        root.join("theme.toml"),
        "[acccent]\nr = 1\ng = 2\nb = 3\nansi16 = 4\n",
    )
    .expect("the theme");

    let output = status(&root);
    assert!(output.status.success());
    assert!(said(&output).contains("acccent"), "{}", said(&output));
}
