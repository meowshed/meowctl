---
id: REQ-3243
artifact: requirement
topic: tui
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3243

A non-UTF-8 locale MUST select the ASCII glyph set.

The alternative was always using Unicode, which lost because replacement
characters would appear where the status glyph should be (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-043` in `docs/spec/tui.md`, its first obligation.
