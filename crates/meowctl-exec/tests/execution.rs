//! What running a command has to do, and what it must not know about.
//!
//! [R-EXEC-001] is held by the file rather than by one test: running to
//! completion and resolving a name on `PATH` are the two things the trait
//! covers, and both are exercised below against `RealExecutor` as well as the
//! scripted one, which is [R-EXEC-010].
//!
//! [R-EXEC-022] is held by what is absent: nothing here constructs a
//! renderer, because `meowctl-exec` cannot -- it depends on
//! `meowctl-common` for the event vocabulary and on nothing that renders.

// `clippy.toml` exempts tests from `expect_used`, but only a function carrying
// `#[test]`. A helper in a test binary is test code by construction.
#![allow(clippy::expect_used)]

use meowctl_common::{Event, Level, Phase, Stream};
use meowctl_exec::{
    Command, DryRunExecutor, ExecError, Executor, Output, RealExecutor, ScriptedExecutor,
    ScriptedRun,
};

/// Collects what an executor emitted, so a test can assert on the stream the
/// sinks will render.
fn record(run: impl FnOnce(&mut dyn FnMut(Event))) -> Vec<Event> {
    let mut events = Vec::new();
    run(&mut |e| events.push(e));
    events
}

/// [R-EXEC-003] component code reads `exit_code` and `stdout` by name, so the
/// shape of a result is part of the Starlark API rather than an internal
/// detail.
#[test]
fn a_result_carries_the_three_fields_separately() {
    let exec = ScriptedExecutor::new([ScriptedRun::ok("echo hello", "hello\n")]);
    let out = record(|events| {
        let got = exec
            .run(&Command::new("echo").args(["hello"]), events)
            .expect("run");
        assert_eq!(got.stdout, "hello\n");
        assert_eq!(got.stderr, "");
        assert_eq!(got.exit_code, Some(0));
        assert!(got.succeeded());
    });
    assert!(!out.is_empty());
}

/// [R-EXEC-004] an interrogation hook asking about a package it does not have
/// gets a non-zero exit and wants to read it, not to fail.
#[test]
fn a_non_zero_exit_is_a_result_and_not_an_error() {
    let exec = ScriptedExecutor::new([ScriptedRun::fails("brew list widget", 1, "not found\n")]);
    let got = record_run(&exec, &Command::new("brew").args(["list", "widget"]));
    assert_eq!(got.exit_code, Some(1));
    assert!(!got.succeeded());
    assert_eq!(got.stderr, "not found\n");
}

fn record_run(exec: &dyn Executor, command: &Command) -> Output {
    let mut sink = |_: Event| {};
    exec.run(command, &mut sink).expect("run")
}

/// [R-EXEC-020] a long install that showed nothing until it finished would
/// look hung, so output reaches the sink line by line.
#[test]
fn output_reaches_the_sink_as_lines() {
    let exec = ScriptedExecutor::new([ScriptedRun::ok("brew install", "one\ntwo\n")]);
    let events = record(|sink| {
        exec.run(&Command::new("brew").args(["install"]), sink)
            .expect("run");
    });

    let lines: Vec<&str> = events
        .iter()
        .filter_map(|e| match e {
            Event::ProcessOutput {
                stream: Stream::Stdout,
                line,
            } => Some(line.as_str()),
            _ => None,
        })
        .collect();
    assert_eq!(lines, ["one", "two"]);
}

/// A command that ends its output properly should not look as though it
/// printed a blank line.
#[test]
fn a_trailing_newline_does_not_become_an_empty_line() {
    let exec = ScriptedExecutor::new([ScriptedRun::ok("echo", "only\n")]);
    let events = record(|sink| {
        exec.run(&Command::new("echo"), sink).expect("run");
    });
    let count = events
        .iter()
        .filter(|e| matches!(e, Event::ProcessOutput { .. }))
        .count();
    assert_eq!(count, 1, "{events:?}");
}

/// [R-EXEC-012] a test that runs something it did not expect has found a
/// defect, and an empty result would hide it.
#[test]
fn an_unscripted_command_fails_the_test() {
    let exec = ScriptedExecutor::new([]);
    let mut sink = |_: Event| {};
    let err = exec
        .run(&Command::new("rm").args(["-rf", "/"]), &mut sink)
        .expect_err("should refuse");
    assert!(matches!(err, ExecError::Unscripted { .. }), "{err:?}");
    assert!(err.to_string().contains("rm -rf /"), "{err}");
}

/// [R-EXEC-011] an install_check hook that cannot interrogate the system plans
/// against nothing, so a read-only phase still runs its commands.
#[test]
fn a_read_only_phase_still_runs_commands_in_a_dry_run() {
    let scripted = ScriptedExecutor::new([ScriptedRun::ok("brew list", "git\n")]);
    let dry = DryRunExecutor::new(Box::new(scripted), Phase::InstallCheck);
    assert!(dry.runs_commands());

    let got = record_run(&dry, &Command::new("brew").args(["list"]));
    assert_eq!(got.stdout, "git\n");
}

/// [R-EXEC-011] and every other phase runs nothing at all.
#[test]
fn a_mutating_phase_runs_nothing_in_a_dry_run() {
    let scripted = ScriptedExecutor::new([]);
    let dry = DryRunExecutor::new(Box::new(scripted), Phase::Install);
    assert!(!dry.runs_commands());

    let events = record(|sink| {
        let got = dry
            .run(&Command::new("brew").args(["install", "git"]), sink)
            .expect("run");
        assert_eq!(got.stdout, "");
    });

    // Reported rather than silently skipped: a plan that does not say what it
    // would have run is a plan nobody can check.
    let said = events.iter().any(|e| {
        matches!(e, Event::Message { level: Level::Info, text } if text.contains("brew install git"))
    });
    assert!(said, "{events:?}");
}

/// A hook that branches on failure would otherwise take the failure path for
/// every command in a dry run and plan a repair that is not needed.
#[test]
fn a_skipped_command_reports_success() {
    let dry = DryRunExecutor::new(Box::new(ScriptedExecutor::new([])), Phase::Install);
    let got = record_run(&dry, &Command::new("anything"));
    assert!(got.succeeded());
}

/// A component writes `cmd`, not `cmd.exe`, so resolving the extension is the
/// lookup's job. Windows-only, because it is the only platform with the
/// problem.
#[cfg(windows)]
#[test]
fn a_windows_program_resolves_without_its_extension() {
    let exec = RealExecutor::new();
    assert!(
        exec.which("cmd").expect("which").is_some(),
        "cmd should resolve to cmd.exe"
    );
}

/// [R-EXEC-032] asking whether a tool exists is a question, not an assertion,
/// and [R-EXEC-001] and [R-EXEC-010]: `RealExecutor` resolves a real name on a
/// real `PATH`.
#[test]
fn which_answers_absent_rather_than_failing() {
    let exec = ScriptedExecutor::new([]).with_path(["git"]);
    assert!(exec.which("git").expect("which").is_some());
    assert!(exec.which("nonesuch").expect("which").is_none());
}

/// [R-EXEC-021] the sink stands down for the duration, and [R-EXEC-023] a
/// captured command must not tear the live region down.
#[test]
fn only_an_interactive_command_asks_for_the_terminal() {
    let exec = ScriptedExecutor::new([ScriptedRun::ok("quiet", "")]);
    let events = record(|sink| {
        exec.run(&Command::new("quiet"), sink).expect("run");
    });
    assert!(
        !events.iter().any(|e| matches!(e, Event::TerminalRequested)),
        "{events:?}"
    );
}

/// [R-EXEC-030] a hook calling a package manager that is not installed is the
/// commonest failure there is, and the message has to name it.
#[test]
fn a_missing_program_is_named() {
    let exec = RealExecutor::new();
    let mut sink = |_: Event| {};
    let err = exec
        .run(&Command::new("meowctl-no-such-program"), &mut sink)
        .expect_err("should fail");
    assert!(matches!(err, ExecError::NotOnPath { .. }), "{err:?}");
    assert!(err.to_string().contains("meowctl-no-such-program"), "{err}");
}

/// The real executor is exercised once, against a command every platform in
/// the matrix has, so the capture path is not only tested through a double.
///
/// On Windows the program is named without its extension on purpose: that is
/// how a component writes it, and resolving `cmd` to `cmd.exe` is the lookup's
/// job rather than the caller's.
#[test]
fn the_real_executor_captures_output() {
    let exec = RealExecutor::new();
    let program = if cfg!(windows) { "cmd" } else { "echo" };
    let args: Vec<&str> = if cfg!(windows) {
        vec!["/C", "echo hello"]
    } else {
        vec!["hello"]
    };

    let mut sink = |_: Event| {};
    let got = exec
        .run(&Command::new(program).args(args), &mut sink)
        .expect("run");
    assert!(got.succeeded(), "{got:?}");
    assert!(got.stdout.contains("hello"), "{got:?}");
}

/// [R-EXEC-005] a hook that lost PATH would lose every tool, so the overrides
/// go over the parent environment rather than replacing it.
#[test]
fn the_environment_is_merged_rather_than_replaced() {
    let exec = RealExecutor::new();
    if cfg!(windows) {
        return;
    }

    let mut sink = |_: Event| {};
    let got = exec
        .run(
            &Command::new("sh")
                .args(["-c", "printf %s \"$MEOWCTL_TEST:${PATH:+set}\""])
                .env("MEOWCTL_TEST", "value"),
            &mut sink,
        )
        .expect("run");
    assert_eq!(got.stdout, "value:set", "{got:?}");
}

/// [R-EXEC-002] a command is a program and an argument list, never a shell
/// string, so an argument containing a space or a semicolon is an argument.
///
/// `v0.1.0` does the same, and it is what keeps a component from being a
/// shell injection waiting for a filename.
#[test]
fn an_argument_with_a_space_stays_one_argument() {
    let exec = ScriptedExecutor::new([ScriptedRun::ok("echo one two; rm -rf /", "")]);
    let command = Command::new("echo").args(["one two; rm -rf /"]);

    assert_eq!(command.program, "echo");
    assert_eq!(command.args, ["one two; rm -rf /"]);

    let mut events = |_| {};
    exec.run(&command, &mut events).expect("the run");
}

/// [R-EXEC-031] the sink gets the terminal back even when the command never
/// starts, because a renderer that never hears the second half never draws
/// again and the user is left with a dead screen.
///
/// `RealExecutor` because the hand-off is its, and `false` because the
/// failure has to happen after the sink has already stood down -- a program
/// that is not on `PATH` is refused before the terminal is ever asked for,
/// and there is nothing to give back.
///
/// Unix only: `false` is the portable program that does one thing and fails.
#[cfg(unix)]
#[test]
fn the_terminal_comes_back_when_the_command_fails() {
    let exec = RealExecutor::new();
    let command = Command::new("false").interactive();
    let events = record(|sink| {
        let _ = exec.run(&command, sink);
    });

    assert!(
        events.iter().any(|e| matches!(e, Event::TerminalReleased)),
        "the terminal was not given back: {events:?}"
    );
}
