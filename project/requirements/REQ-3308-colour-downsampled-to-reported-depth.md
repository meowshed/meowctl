---
id: REQ-3308
artifact: requirement
topic: tui
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3308

Colour MUST be downsampled to the depth the terminal reports rather than
assumed.

The alternative was assuming truecolor, which lost because that produces garbage
on a 16-colour terminal (from docs/design/0.2.0-requirement-tradeoffs.md, high).
The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-042` in `docs/spec/tui.md`, its second obligation.
