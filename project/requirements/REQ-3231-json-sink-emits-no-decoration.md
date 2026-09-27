---
id: REQ-3231
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3231

`JsonSink` MUST emit no colour, no glyph, and no padding.

`JsonSink` is the interface another program reads; see REQ-1043 (from
docs/spec/tui.md, high). The alternative was including the rendered line, which
lost because a consumer would parse terminal decoration (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-031` in `docs/spec/tui.md`.
