---
id: REQ-3253
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3253

The theme file MUST be `theme.toml` in the configuration directory.

The alternative was a file of its own under `$XDG_CONFIG_HOME`, which lost
because a user who moves their configuration between machines expects their
colours to come with it (from docs/design/0.2.0-requirement-tradeoffs.md, high).
The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-053` in `docs/spec/tui.md`, its first obligation.
