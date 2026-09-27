---
id: REQ-3220
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3220

`LiveSink` MUST redraw its region in place.

`internal/tui/live.go` in `v0.1.0` is the behaviour, including capping the
region to the viewport so the arithmetic cannot exceed it (from
docs/spec/tui.md, high). The alternative was append-only output everywhere,
which lost because the live renderer is the reason progress is readable on a
terminal (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-020` in `docs/spec/tui.md`, its first obligation.
