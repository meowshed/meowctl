---
id: REQ-3221
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3221

Captured subprocess output MUST appear under its component while the process
runs.

The alternative was showing output after the process exits, which lost because a
long install would show nothing while it runs (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-021` in `docs/spec/tui.md`.
