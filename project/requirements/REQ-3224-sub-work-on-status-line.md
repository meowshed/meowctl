---
id: REQ-3224
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3224

Sub-work within a component, such as packages being installed, MUST be shown on
the component's own status line rather than by indenting further, so REQ-3202
holds.

The alternative was indenting sub-work, which lost because it breaks the
two-indent rule that makes output scannable (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if sub-work
becomes important enough to justify a third level (from
docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-024` in `docs/spec/tui.md`.
