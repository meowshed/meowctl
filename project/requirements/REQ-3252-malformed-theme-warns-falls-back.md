---
id: REQ-3252
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3252

A theme file that is malformed MUST warn and fall back to the default, not fail
the command.

Nobody's apply should stop because their colours are wrong (from
docs/spec/tui.md, high). The alternative was failing on a bad theme, which lost
because nobody's apply should stop over colours (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-052` in `docs/spec/tui.md`.
