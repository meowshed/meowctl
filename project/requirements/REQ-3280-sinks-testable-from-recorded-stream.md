---
id: REQ-3280
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3280

Every sink MUST be testable against a recorded event stream with no terminal, no
subprocess, and no filesystem.

The alternative was testing through the engine, which lost because every
rendering test would need a run (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-080` in `docs/spec/tui.md`.
