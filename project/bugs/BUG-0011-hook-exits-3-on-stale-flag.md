---
id: BUG-0011
artifact: bug
status: approved
severity: major
violates: REQ-3522
found: 2026-09-27
revised: 2026-09-27
issue: 147
---

# `hook` exits 3 and prints an error on every shell spawn when a stale `.hook-error` can't be removed

After a successful hook run, `hook` propagates a failure to remove
`.hook-error`, so the exit code becomes 3.

## Reproduction

The research ran the debug binary `target/debug/meowctl`, built from revision
`7e8cabe`, on macOS in a scratch directory, with `HOME`, `XDG_CACHE_HOME` and
`--config` pointed there. The research left a stale `.hook-error` in the
configuration directory, set the directory to mode 555, and ran `meowctl hook
shell`.

## What the system does

It printed `echo hi`, then `meowctl: removing .../.hook-error: Permission denied
(os error 13)`, and exited 3. `HookError::clear(...)?` propagates the removal
failure (`crates/meowctl-cli/src/run/commands.rs:541-543`). Every shell spawn
repeats it.

## What it should do, and why

[REQ-3522] says the exit code stays 0, and [REQ-3542] says `hook` reports the
failed removal on stderr, naming the path. The fix turns the `?` at
`commands.rs:542` into a reported warning.

## Triage

A requirement covers it, so the fix enters at implement. It is major because
every shell start prints an error, which is the outcome [REQ-3522] exists to
prevent.

## Closed by

Open.
