---
id: REQ-3251
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3251

Call sites MUST name a role, never a colour, so the palette changes in one
place.

The alternative was naming colours at call sites, which lost because changing
the palette means touching every call site (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-051` in `docs/spec/tui.md`.
