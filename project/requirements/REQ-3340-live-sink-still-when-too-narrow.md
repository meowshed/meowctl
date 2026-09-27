---
id: REQ-3340
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3340

The live sink MUST NOT move the cursor while the terminal is narrower than its
minimum width.

A frame laid out wider than the terminal wraps, and the cursor-up arithmetic of
the next frame then redraws over the wrong lines and corrupts the scrollback.

Added during onboarding from the failure-path research; the shipped sink lays
out a terminal narrower than 20 columns at 80 (from
crates/meowctl-tui/src/live.rs:35-39 and live.rs:230-236, high).
