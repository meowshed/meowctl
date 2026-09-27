---
id: SPC-1600
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-1601, REQ-1602, REQ-1603, REQ-1604, REQ-1605, REQ-1606, REQ-1610, REQ-1611, REQ-1612, REQ-1620, REQ-1621, REQ-1622, REQ-1623, REQ-1630, REQ-1631, REQ-1632, REQ-1700, REQ-1701, REQ-1702, REQ-1703, REQ-1704, REQ-1770, REQ-2941]
---

# Process execution

## Scope

This component owns every subprocess the workspace spawns, behind one trait. It
also owns the hand-off of the terminal to an interactive subprocess, which in
`v0.1.0` was a callback threaded from the renderer into the Starlark layer (from
docs/spec/exec.md, high).

It lives in the crate `meowctl-exec`, its design is
`docs/design/0.2.0-rust-rewrite.md` §3 and defect #11, and its `v0.1.0`
equivalent is `execRun`, `starRun`, `starWhich` and `starGitClone` in
`internal/ctx/methods.go` (from docs/spec/exec.md, high). `git_clone` isn't an
executor concern: it's a `ctx` method that builds a `git` command, which the
`ctx` specification covers (from docs/spec/exec.md, high).

## Boundary

The `Executor` trait, the shape of a command and its result, and the events
emitted around a process (from docs/spec/exec.md, high).

| Surface | What it is |
| --- | --- |
| `Executor` | Runs a command to completion and resolves a name on `PATH` (from docs/spec/exec.md, high) |
| `RealExecutor`, `DryRunExecutor`, the test executor | The three implementations (from docs/spec/exec.md, high) |
| `ProcessOutput`, `TerminalRequested`, `TerminalReleased` | The events a process emits (from docs/spec/exec.md, high) |

## Behaviour

### The trait

`Executor` covers running a command to completion and resolving a command name
on `PATH`, and nothing else spawns a process [REQ-1601] (from docs/spec/exec.md,
high). On Windows, resolving a name also tries each extension `PATHEXT` names,
in its order, and falls back to `.COM;.EXE;.BAT;.CMD` when `PATHEXT` is unset;
on every other platform the name is the file [REQ-1606] (from docs/spec/exec.md,
high).

A command is a program name and an argument list, never a shell string
[REQ-1602] (from docs/spec/exec.md, high). The result carries stdout, stderr and
the exit code separately [REQ-1603] (from docs/spec/exec.md, high), and a
non-zero exit is a result the caller inspects, not an error [REQ-1604] (from
docs/spec/exec.md, high). The process sees the parent environment merged with
the per-call overrides, and an override wins [REQ-1605] (from docs/spec/exec.md,
high).

### Implementations

`RealExecutor` spawns the process [REQ-1610] (from docs/spec/exec.md, high).
`DryRunExecutor` runs a command issued from a read-only phase [REQ-1611] and
doesn't run one issued from any other phase [REQ-1700]; the phase decides, and
the read-only phases are the ones REQ-1012 names (from docs/spec/exec.md, high).
A command it doesn't run returns exit code 0 with empty standard output and
standard error [REQ-1770], so a hook that branches on failure doesn't plan a
repair that isn't needed (from https://github.com/meowshed/meowctl/pull/58,
high).
The test executor answers a scripted set of commands [REQ-1612] and fails the
test when an unscripted command runs [REQ-1701] (from docs/spec/exec.md, high).

### Output and the terminal

A process's output reaches the caller as `ProcessOutput` events carrying the
stream and the line, and is also accumulated for the result [REQ-1620] (from
docs/spec/exec.md, high). An interactive command emits `TerminalRequested`
before it starts and `TerminalReleased` after it exits, and the sink stands down
in between [REQ-1621] (from docs/spec/exec.md, high). A command that isn't
interactive never requests the terminal, and its output is captured and rendered
under its component [REQ-1623] (from docs/spec/exec.md, high).

`meowctl-exec` doesn't know what a renderer is [REQ-1622], holds none [REQ-1702]
and takes no callback that suspends one [REQ-1703] (from docs/spec/exec.md,
high). `SuspendOutput` exists nowhere in the tree [REQ-1704] (from
docs/spec/exec.md, high).

## Failure paths

| Condition | What happens |
| --- | --- |
| The command doesn't exist | A distinct error names the command, in place of a generic spawn failure [REQ-1630] (from docs/spec/exec.md, high) |
| The command exits non-zero | The call succeeds and the result carries the exit code [REQ-1604] (from docs/spec/exec.md, high) |
| A process is killed by a signal | The result has no exit code; `ctx.run` reports it as `-1` and doesn't raise, and an `Op` that runs the process fails [REQ-1604] [REQ-2941] (from crates/meowctl-exec/src/real.rs:64 and crates/meowctl-ctx/src/value.rs:262-280, high) |
| The command can't be started, or its output can't be read | The call returns an error [REQ-1604] (from docs/spec/exec.md, high) |
| An interactive process fails, or the run is interrupted | `TerminalReleased` is still emitted [REQ-1631] (from docs/spec/exec.md, high) |
| No file on `PATH` matches a name | Resolution returns an absent result, not an error [REQ-1632] (from docs/spec/exec.md, high) |
| A dry run meets a command from a phase that isn't read-only | `DryRunExecutor` doesn't run it [REQ-1700] (from docs/spec/exec.md, high) |
| A test runs an unscripted command | The test executor fails the test [REQ-1701] (from docs/spec/exec.md, high) |
