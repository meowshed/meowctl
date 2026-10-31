---
id: TSK-0045
artifact: task
status: done
revised: 2026-10-31
bug: BUG-0001
closes: []
---

# Empty the journal after a successful run

`meowctl apply` truncates `rollback.jsonl` when the run neither failed nor was
interrupted, so a later failed run undoes only its own work.

## Acceptance criteria

1. Given a configuration whose component writes a file, when `apply` succeeds,
   then `rollback.jsonl` holds nothing. Closed by:
   `a_successful_run_leaves_an_empty_journal` in `tests/journal.rs`.
2. Given an earlier successful run that wrote `~/a.txt`, when a later run adds
   a component that writes `~/b.txt` and fails, then `~/a.txt` is still there
   and `~/b.txt` isn't. Closed by:
   `a_failed_run_does_not_undo_an_earlier_successful_one` in
   `tests/journal.rs`.

## What to do

In `apply` in `crates/meowctl-cli/src/run/commands.rs`, keep a handle on the
journal the effects carry, and call `Journal::truncate` once the run has
neither failed nor been interrupted, restoring REQ-2025.

## Depends on

None.

## Left alone

Emptying the journal after a completed rollback, which BUG-0001 mentions, has
no requirement yet, so a rolled-back run still leaves its records.

## Evidence

Criterion 1 is closed by `a_successful_run_leaves_an_empty_journal` and
criterion 2 by `a_failed_run_does_not_undo_an_earlier_successful_one` in
`tests/journal.rs`, which drive the binary with `HOME` in a sandbox. Both failed
before the change, the second on "the failed run undid the earlier one", and
pass after it. `mise run fmt-check`, `mise run check`, `mise run test` (610
passed, 3 skipped) and `mise run build` exit 0.
