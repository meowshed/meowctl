---
id: BUG-0018
artifact: bug
status: approved
severity: major
violates: REQ-2140
found: 2026-09-27
revised: 2026-09-27
issue: 154
---

# An unwritable journal is found at the first reversible effect, after irreversible steps have run

`Journal::open` only reads, and the first `append` creates the file, so a
configuration directory that isn't writable is found only when a hook makes its
first reversible effect.

## Reproduction

The onboarding research inferred this from the code at `main` as #97 left it and
didn't run it. It agrees with the `.hook-error` probe in BUG-0012, where a
directory at mode 555 refused writes. At `main` as #97 left it, set the
configuration directory to mode 555 and run `meowctl apply` with a component
whose `install` hook calls `ctx.run` and then writes a file.

## What the system does

`Journal::open` treats an absent file as an empty journal
(`crates/meowctl-ops/src/journal.rs:87-100`), and `append` creates it with
`OpenOptions::create(true).append(true)` (`journal.rs:155-162`). The `ctx.run`
step runs, the write fails with `CtxError::Effect`, `replay` finds no journal
and undoes nothing, and the command exits 1. `interrupted_run` treats an
unreadable journal as absent (`crates/meowctl-engine/src/progress.rs:174-179`),
so no warning appears at startup. The journal uses `std::fs` directly rather
than the `FileSystem` passed in (`journal.rs:155`), which is why no in-memory
test reaches this case.

## What it should do, and why

[REQ-2140] says a run that journals doesn't start its first phase unless its
journal can be written, and SPC-2000 says the run stops naming the path.

## Triage

A requirement covers it, so the fix enters at implement. It is major because
every non-reversible step before the first reversible one, such as a package
install, has already run, with nothing to undo it.

## Closed by

Open.
