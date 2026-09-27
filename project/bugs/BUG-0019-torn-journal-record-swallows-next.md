---
id: BUG-0019
artifact: bug
status: approved
severity: minor
violates: REQ-2141
found: 2026-09-27
revised: 2026-09-27
issue:
---

# A torn journal record swallows the record appended after it

A partial `writeln` leaves a line with no newline, and the next append joins its
record onto that line, so replay skips both.

## Reproduction

The onboarding research inferred this from the code at revision `7e8cabe` and
didn't run it. At revision `7e8cabe`, truncate the last byte of a record in
`rollback.jsonl`, run a component that makes one reversible effect and fails,
and read the rollback output.

## What the system does

`append` writes after whatever the file ends with
(`crates/meowctl-ops/src/journal.rs:155-162`), so the new record shares a line
with the torn one. Replay skips an unparseable line [REQ-2030], and the second
record's inverse is lost. `append` also increments `seq` before the write, so a
failed append leaves a gap in the numbering, which does no harm. Because of
BUG-0001 the journal outlives the run, so the next run reaches this state.

## What it should do, and why

[REQ-2141] says a record stays readable on its own when the record before it was
torn. One fix is to start each append with a newline when the file doesn't end
in one.

## Triage

A requirement covers it, so the fix enters at implement. It is minor because it
needs a torn write first, which takes a crash or a full disk during an append.

## Closed by

Open.
