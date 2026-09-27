---
id: REQ-3225
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3225

The terminal width MUST be re-read per frame, so a resize is picked up without a
signal handler.

`Caps.Size` does this (from docs/spec/tui.md, high). The alternative was reading
the width once, which lost because a resize mid-apply corrupts every frame after
it (from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a
signal handler is added (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-025` in `docs/spec/tui.md`.
