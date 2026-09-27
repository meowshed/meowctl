---
id: BUG-0001
artifact: bug
status: approved
severity: critical
violates: REQ-2025
found: 2026-09-27
revised: 2026-09-27
issue:
---

# The rollback journal is never truncated, so a failed run undoes an earlier successful run

`Journal::truncate` exists and nothing outside the tests calls it, so the
journal keeps every run's records and the next rollback replays all of them.

## Reproduction

The research ran the debug binary `target/debug/meowctl`, built from revision
`7e8cabe`, on macOS in a scratch directory, with `HOME`, `XDG_CACHE_HOME` and
`--config` pointed there. The research ran these steps:

1. Declare a component `a` whose `install` hook writes `~/a.txt` through `ctx`.
2. Run `meowctl apply`. It exits 0, and `rollback.jsonl` still holds the record
   for `~/a.txt`.
3. Run `meowctl apply` again. It prints `i a previous run stopped partway and
   left 1 operation(s) to undo`.
4. Add a component `b` whose `install` hook writes `~/b.txt` and then calls
   `fail("boom")`, and run `meowctl apply`.

## What the system does

The third run rolls back and prints `remove .../home/b.txt` and `remove
.../home/a.txt`. It deletes the file the first, successful run wrote, while
`state.toml` still marks `a` as done, so the next `apply` skips `a` and never
puts the file back. Every run after the first also warns of an interrupted run
that didn't happen.

`Journal::truncate` is defined at `crates/meowctl-ops/src/journal.rs:227-244`,
and `grep -rn "truncate" crates/*/src` finds no caller in
`crates/meowctl-cli/src/run/commands.rs:460-498` or anywhere else outside the
tests. The test comment at `crates/meowctl-ops/tests/journal.rs:199` names this
outcome: "every run would otherwise replay the last one's operations".

## What it should do, and why

[REQ-2025] says the journal is truncated after a successful run. The fix also
empties it once a rollback completes, which no requirement states yet, because a
rolled-back run otherwise leaves records a later rollback would replay.

## Triage

A requirement covers it, so the fix enters at implement. It is critical because
a failure in one component deletes files that an earlier successful run
installed, and `state.toml` then records them as installed, so no later run
repairs the damage.

## Closed by

Open. The fix adds a binary-level test that runs `apply` twice and then fails a
third run, and checks that the first run's file survives.
