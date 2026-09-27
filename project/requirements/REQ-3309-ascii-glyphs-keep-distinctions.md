---
id: REQ-3309
artifact: requirement
topic: tui
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3309

The ASCII glyph set MUST carry the same distinctions as the Unicode one.

The alternative was always using Unicode, which lost because replacement
characters would appear where the status glyph should be (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-043` in `docs/spec/tui.md`, its second obligation.
