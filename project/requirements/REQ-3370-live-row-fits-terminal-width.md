---
id: REQ-3370
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3370

A row the live sink draws MUST NOT exceed the terminal width.

A row that exceeds the width wraps, and a wrapped row breaks the cursor-up
arithmetic the next frame uses to redraw the region, so the width has to be
measured in the columns a terminal renders, where a wide character takes two
(from https://github.com/meowshed/meowctl/pull/71 and
crates/meowctl-tui/src/live.rs:458-466, high).

Added during onboarding from the research on decision 2; the shipped
`display_width` counts characters, so a wide character can push a row past the
width, which BUG-0021 records (from crates/meowctl-tui/src/live.rs:458-466,
high).
