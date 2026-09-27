---
id: REQ-3254
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3254

A role the theme file does not name MUST keep its default, so a user who wants
one colour changed writes one table.

The alternative was mixing a named role's missing numbers with the default,
which lost because three numbers and a fourth chosen for you is a colour nobody
picked (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-054` in `docs/spec/tui.md`, its first obligation.
