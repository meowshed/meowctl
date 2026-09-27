---
id: REQ-3222
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3222

On `TerminalRequested`, `LiveSink` MUST erase its region, restore the cursor,
and write nothing until `TerminalReleased`; see REQ-1621.

The alternative was drawing on during a subprocess, which lost because that
makes two writers to one terminal (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-022` in `docs/spec/tui.md`.
