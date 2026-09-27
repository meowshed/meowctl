---
id: REQ-3230
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3230

`JsonSink` MUST emit one JSON object per event, one per line, in the order the
events arrived.

The alternative was one JSON document at the end, which lost because a consumer
could not stream, and a killed run would emit nothing (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-030` in `docs/spec/tui.md`.
