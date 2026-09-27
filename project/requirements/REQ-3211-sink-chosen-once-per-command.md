---
id: REQ-3211
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3211

A sink MUST be chosen once per command, from the detected capabilities and the
flags.

The alternative was re-detecting per event, which lost because output would
change shape mid-run (from docs/design/0.2.0-requirement-tradeoffs.md, high).
The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-011` in `docs/spec/tui.md`, its first obligation.
