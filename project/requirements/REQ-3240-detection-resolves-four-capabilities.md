---
id: REQ-3240
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3240

Detection MUST resolve four things independently: whether the destination is a
terminal, whether motion is allowed, whether the locale advertises UTF-8, and
the colour depth.

The alternative was one "is a terminal" flag, which lost because a CI job with a
pty, a UTF-8 pipe, and a 16-colour terminal each need a different answer (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-040` in `docs/spec/tui.md`.
