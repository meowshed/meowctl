---
id: REQ-1043
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1043

`Event` MUST NOT carry a rendered string, a colour, a glyph, or a width.

Those are the sink's decisions, and an event that carries one makes the JSON
sink emit terminal decoration (from docs/spec/common.md, high). The alternative
was letting the engine pick a glyph, which lost because then `JsonSink` emits
terminal decoration and the engine holds a theme, and the record expects nothing
to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-043` in `docs/spec/common.md`.
