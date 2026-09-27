---
id: BUG-0009
artifact: bug
status: approved
severity: minor
violates: REQ-3340
found: 2026-09-27
revised: 2026-09-27
issue: 145
---

# A terminal narrower than 20 columns is laid out at 80, so every live frame wraps

`LiveSink::width` replaces any width under `MINIMUM_WIDTH` (20) with
`FALLBACK_WIDTH` (80), so on a narrow terminal each line wraps and the next
frame redraws over the wrong lines.

## Reproduction

The onboarding research inferred this from the code at revision `7e8cabe` and
didn't run it. At revision `7e8cabe`, resize a terminal to 10 columns and run
`meowctl apply` with the live sink.

## What the system does

The width substitution is at `crates/meowctl-tui/src/live.rs:35-39` and
`live.rs:230-236`. The comment at `live.rs:461` says a wrapped line breaks the
cursor-up arithmetic of the next frame (medium: from the code and its comment,
not probed). Nothing panics, because budgets use `saturating_sub`. The snapshot
matrix covers only widths 80 and 30 (`crates/meowctl-tui/tests/live.rs:353`).

## What it should do, and why

[REQ-3340] says the live sink doesn't move the cursor while the terminal is
narrower than its minimum width. The fix prints lines as the plain sink does
below the minimum.

## Triage

A requirement covers it, so the fix enters at implement. It is minor because it
corrupts only the display, and a terminal under 20 columns is rare.

## Closed by

Open. The fix adds a snapshot at width 10.
