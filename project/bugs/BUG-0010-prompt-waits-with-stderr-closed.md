---
id: BUG-0010
artifact: bug
status: approved
severity: minor
violates: REQ-3341
found: 2026-09-27
revised: 2026-09-27
issue: 146
---

# With stderr closed at startup, a prompt waits for an answer to a question nobody saw

A session counts as interactive when stdin is a terminal, whatever stderr is, so
a prompt writes its question to `/dev/null` and then blocks on stdin.

## Reproduction

The onboarding research inferred this from the code at `main` as #97 left it and
didn't run it. It rests on Rust standard library behaviour on Unix, which
reopens descriptors 0 to 2 on `/dev/null` at startup. At `main` as #97 left it,
run `meowctl apply 2>&-` in a terminal with a component whose hook calls
`ctx.prompt`.

## What the system does

`Prompt::ask` writes the question to stderr, which succeeds against `/dev/null`,
then reads stdin (`crates/meowctl-tui/src/interaction.rs:81-137`). Interactivity
is decided from stdin alone (`crates/meowctl-cli/src/run.rs:257-261`). The
process waits with nothing on screen.

## What it should do, and why

[REQ-3341] says a prompt doesn't wait for input unless its question was written
to a terminal. The fix treats a stderr that isn't a terminal as non-interactive,
so the prompt fails as [REQ-3262] describes.

## Triage

A requirement covers it, so the fix enters at implement. It is minor because it
needs stderr closed or redirected while stdin stays a terminal, and an interrupt
ends the wait.

## Closed by

Open.
