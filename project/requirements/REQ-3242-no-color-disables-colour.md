---
id: REQ-3242
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3242

`NO_COLOR` MUST disable colour.

The alternative was assuming truecolor, which lost because that produces garbage
on a 16-colour terminal (from docs/design/0.2.0-requirement-tradeoffs.md, high).
The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-042` in `docs/spec/tui.md`, its first obligation.
