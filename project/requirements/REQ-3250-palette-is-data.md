---
id: REQ-3250
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3250

The palette MUST be data rather than colours at call sites, with the Catppuccin
values as the built-in default.

The alternative was compiling the palette in, which lost because a user whose
terminal clashes with Catppuccin has no recourse (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-050` in `docs/spec/tui.md`, its first obligation.
