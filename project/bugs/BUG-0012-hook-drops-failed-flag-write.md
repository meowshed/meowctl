---
id: BUG-0012
artifact: bug
status: approved
severity: major
violates: REQ-3542
found: 2026-09-27
revised: 2026-09-27
issue: 148
---

# `hook` drops a failed write of `.hook-error` silently

`hook` writes the flag with `let _ = flag.write(...)`, so when the write fails
nobody learns that a hook failed.

## Reproduction

The research ran the debug binary `target/debug/meowctl`, built from revision
`7e8cabe`, on macOS in a scratch directory, with `HOME`, `XDG_CACHE_HOME` and
`--config` pointed there. The research set the configuration directory to mode
555, declared a component whose `shell` hook fails, and ran `meowctl hook
shell`.

## What the system does

It exited 0, printed nothing and left no `.hook-error`
(`crates/meowctl-cli/src/run/commands.rs:541-558`). `status` and `doctor` then
report nothing, so [REQ-3464] can't hold.
`crates/meowctl-config/tests/hookerr.rs` covers the file format and `clear` on
an absent file, not a failing write.

## What it should do, and why

[REQ-3542] says `hook` reports on stderr when it can't record `.hook-error`,
keeping exit 0 [REQ-3522].

## Triage

A requirement covers it, so the fix enters at implement. It is major because the
hook failure disappears: the flag is the only record of it, and `status` and
`doctor` show a healthy machine.

## Closed by

Open.
