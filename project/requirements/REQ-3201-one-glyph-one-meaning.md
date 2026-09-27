---
id: REQ-3201
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3201

One glyph MUST mean the same thing in every command.

A component that installed, a check that passed, and a dependency that resolved
carry the same success mark (from docs/spec/tui.md, high). The alternative was
per-command vocabularies, which lost because `v0.1.0` had four before the design
system, and a glyph meant different things in each (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-001` in `docs/spec/tui.md`.
