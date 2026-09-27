---
id: REQ-3212
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3212

Every sink that renders a run MUST render every event.

A sink that silently drops a variant makes a command's output depend on where it
runs (from docs/spec/tui.md, high). The alternative was letting a sink drop what
it cannot show, which lost because a command's output would depend on where it
runs (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-012` in `docs/spec/tui.md`, its first obligation.
