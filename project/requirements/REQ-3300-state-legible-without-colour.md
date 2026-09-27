---
id: REQ-3300
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3300

Every state MUST be legible from its symbol and its wording alone, so losing
colour loses decoration and never information.

The alternative was encoding state in colour, which lost because a pipe,
`NO_COLOR`, or a monochrome terminal would lose information (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-003` in `docs/spec/tui.md`, its second obligation.
