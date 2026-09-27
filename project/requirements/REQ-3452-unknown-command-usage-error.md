---
id: REQ-3452
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3452

An unknown command or flag MUST exit 2 with the usage error.

The alternative was to exit 1, which lost because scripts tell a usage mistake
apart from a failure, and the trade-off record names no condition that reverses
it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-052` in `docs/spec/cli.md`, its first obligation.
