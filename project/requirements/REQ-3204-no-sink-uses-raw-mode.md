---
id: REQ-3204
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3204

A sink MUST NOT put the terminal into raw mode.

Hooks shell out to commands that need the real terminal, and the renderer's job
is to stand down rather than to own it (from docs/spec/tui.md, high). The
alternative was taking raw mode for a better renderer, which lost because hooks
shell out to commands that need the real terminal (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if hooks stop
being able to run interactive commands (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-004` in `docs/spec/tui.md`.

Onboarding reworded the source's "No ... MUST" to MUST NOT, because the record's
check refuses a negated subject; the meaning is unchanged.
