---
id: REQ-3315
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3315

A role the theme file names MUST be replaced whole.

A partial colour is four numbers with one missing, and guessing which default to
mix in produces a colour nobody chose (from docs/spec/tui.md, high). The
alternative was mixing a named role's missing numbers with the default, which
lost because three numbers and a fourth chosen for you is a colour nobody picked
(from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table
names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-054` in `docs/spec/tui.md`, its second obligation.
