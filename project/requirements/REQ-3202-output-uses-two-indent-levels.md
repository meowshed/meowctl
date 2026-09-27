---
id: REQ-3202
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3202

Output MUST use at most two levels of indent: column zero for the command
speaking about itself, two spaces for an item, and captured subprocess output
indented under its item and dimmed.

The alternative was nesting to the structure's depth, which lost because the
component graph is deep, and output that mirrors it stops being scannable (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-002` in `docs/spec/tui.md`.
