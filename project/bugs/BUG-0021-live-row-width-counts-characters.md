---
id: BUG-0021
artifact: bug
status: approved
severity: minor
violates: REQ-3370
found: 2026-09-27
revised: 2026-09-27
issue:
---

# The live sink counts characters, so a row holding a wide character can wrap

`display_width` in `crates/meowctl-tui/src/live.rs:458-466` returns
`text.chars().count()`, which counts a CJK character or most emoji as one
column where the terminal renders two, so `truncate` lets a row exceed the
terminal width (from that file, medium).

## Reproduction

Seen at revision `7e8cabe`. The research read the code and didn't run it on a
wide string:

1. Give a component a name or a row note holding a wide character, for
   example `設定`, long enough to reach the terminal width.
2. Run `meowctl apply` in a capable terminal, so `LiveSink` draws the row.

`truncate` measures the name and `row.note` with `display_width` (from
crates/meowctl-tui/src/live.rs:199-208, high).

## What the system does

Each wide character counts as one column, so the measured width is an
under-estimate, the row is drawn wider than the budget, and it wraps. The next
frame moves the cursor up by the number of rows it drew, not the number of
lines on screen, so the redraw lands one line short (inferred from
crates/meowctl-tui/src/live.rs, medium). The doc comment says the count "has to
be an under-estimate at worst", which inverts the reasoning: only an
over-estimate keeps a row from wrapping (from
crates/meowctl-tui/src/live.rs:458-466, high).

## What it should do, and why

No row the live sink draws exceeds the terminal width, REQ-3370, because a
wrapped row breaks the cursor-up arithmetic of the next frame (from
https://github.com/meowshed/meowctl/pull/71, high). Counting an East Asian wide
character as two columns meets it, for example with `unicode-width`, already in
`Cargo.lock` as a transitive dependency, and the comment should say "an
over-estimate at worst" (from Cargo.lock, high).

## Triage

It enters at the live sink's width measurement. Minor, because it corrupts
only the live display for names or notes holding wide characters, which a
component path rarely does, and the plain and JSON sinks are unaffected (from
crates/meowctl-tui/src/live.rs, medium). REQ-3370 was added during onboarding,
so the requirement is a draft.

## Closed by

Open. A test in `crates/meowctl-tui/tests/live.rs` that draws a row holding a
wide character at a fixed width and asserts no line exceeds it.
