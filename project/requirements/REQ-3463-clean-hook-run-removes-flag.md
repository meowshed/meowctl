---
id: REQ-3463
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3463

A run in which nothing failed MUST remove `.hook-error`.

The alternative was to leave the flag until something clears it, which lost
because a fixed configuration would keep warning, and a warning that is always
there is read as noise; the trade-off record names no condition that reverses it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-063` in `docs/spec/cli.md`.
