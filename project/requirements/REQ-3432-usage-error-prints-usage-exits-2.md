---
id: REQ-3432
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3432

A usage error MUST print usage and exit 2.

The alternative was to always print usage on error, which lost because a wall of
flags buries the real error, and the trade-off record names no condition that
reverses it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-032` in `docs/spec/cli.md`, its first obligation.
