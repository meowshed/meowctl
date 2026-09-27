---
id: REQ-3260
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3260

Prompting MUST go through an `Interaction` trait, separate from rendering.

The alternative was prompting from the printer, which lost because that puts
input and output in one type, which is what made `Confirm` hard to test (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-060` in `docs/spec/tui.md`.
