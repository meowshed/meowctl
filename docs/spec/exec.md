# Process execution

**Crate:** `meowctl-exec`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3, defect #11
**v0.1.0 equivalent:** `execRun`, `starRun`, `starWhich`, `starGitClone` in `internal/ctx/methods.go`

## Scope

This component owns every subprocess the workspace spawns, behind one trait.
It also owns the hand-off of the terminal to an interactive subprocess, which
in `v0.1.0` was a callback threaded from the renderer into the Starlark layer.

## Boundary

The `Executor` trait, the shape of a command and its result, and the events
emitted around a process.

## The trait

**[R-EXEC-001]** `Executor` MUST cover running a command to completion and
resolving a command name on `PATH`. Nothing else spawns a process.

**[R-EXEC-002]** A command MUST be built from a program name and an argument
list, never from a shell string. `v0.1.0` does the same, and it is what keeps a
component's arguments from being reinterpreted by a shell that the component
author did not know was there.

**[R-EXEC-003]** The result MUST carry stdout, stderr, and the exit code
separately. `v0.1.0` returns a struct with exactly these three fields from
`ctx.run`, and component code branches on `exit_code`, so the shape is load
bearing; see [R-CTX-020].

**[R-EXEC-004]** A non-zero exit MUST NOT be an error. It is a result the
caller inspects. An error is reserved for a command that could not be started
or whose output could not be read.

**[R-EXEC-005]** The environment MUST be the parent environment merged with
per-call overrides, with the override winning. `mergeRunEnv` in
`internal/ctx/methods.go` is the behaviour.

## Implementations

**[R-EXEC-010]** `RealExecutor` MUST spawn the process.

**[R-EXEC-011]** `DryRunExecutor` MUST run a command issued from a read-only
phase and MUST NOT run one issued from any other phase. The read-only phases
are in [R-COMMON-012], and the phase is what decides, because an
`install_check` hook that cannot ask `brew list` what is installed reports a
plan built on nothing. `validCheckPhases` in `internal/ctx/methods.go` is the
same set for the same reason.

**[R-EXEC-012]** A test executor MUST be able to answer a scripted set of
commands and MUST fail the test when an unscripted command is run. `v0.1.0`
offers `RunFunc` for exactly this, and it is the only injection point the Go
tree has.

## Output and the terminal

**[R-EXEC-020]** A process's output MUST reach the caller as `ProcessOutput`
events carrying the stream and the line, in addition to being accumulated for
the result. A sink decides how to show it; see [R-TUI-021].

**[R-EXEC-021]** An interactive command MUST emit `TerminalRequested` before it
starts and `TerminalReleased` after it exits. The sink stands down for the
duration.

**[R-EXEC-022]** `meowctl-exec` MUST NOT know what a renderer is, MUST NOT hold
one, and MUST NOT take a callback that suspends one. This is defect #11: in
`v0.1.0` the hand-off is `SuspendOutput func() (resume func())`, passed into
`ctx.Capabilities` so that a Starlark builtin can reach back into the terminal
renderer. The events replace it, and `SuspendOutput` MUST NOT exist anywhere in
the tree.

**[R-EXEC-023]** A command that is not interactive MUST NOT request the
terminal. Its output is captured and rendered under its component.

## Failure paths

**[R-EXEC-030]** A command that does not exist MUST produce a distinct error
naming it, not a generic spawn failure. A hook that calls a package manager
that is not installed is the common case.

**[R-EXEC-031]** `TerminalReleased` MUST be emitted even when the process fails
or the run is interrupted. A renderer that never reclaims the terminal leaves
the user without a cursor.

**[R-EXEC-032]** Resolving a name on `PATH` MUST return an absent result rather
than an error when nothing matches, because `ctx.which` is how a component asks
whether a tool is present; see [R-CTX-021].

## Parity with v0.1.0

The result shape, the environment merge, and the argument-list construction are
reproduced from `internal/ctx/methods.go`. `git_clone` is not an executor
concern here: it is a `ctx` method that builds a `git` command, which is what
`starGitClone` already does.

[R-EXEC-011] restates as an obligation what `v0.1.0` implements as a check
inside `starRun`. The behaviour is the same; the difference is that here the
phase is consulted when the executor is constructed rather than on every call,
so no method below `meowctl-cli` branches on a dry run; see [R-CTX-014].
