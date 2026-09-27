---
id: REQ-3342
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3342

When the terminal reports a zero or unknown width, the live sink MUST keep
laying out at a width it last knew.

A zero width is what a terminal reports in the middle of a resize, and laying a
frame out at zero columns would print nothing useful.

Added during onboarding from the failure-path research; the shipped sink falls
back to the width it read at startup, because `terminal_size` returns nothing
for a zero width (from crates/meowctl-tui/src/caps.rs:143-150, high).
