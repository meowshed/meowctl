---
id: REQ-3420
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3420

Every command MUST write through the sink.

The alternative was to print directly where convenient, which lost because a
stray print corrupts the live region, and the trade-off record names no
condition that reverses it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CLI-020` in `docs/spec/cli.md`, its first obligation.
