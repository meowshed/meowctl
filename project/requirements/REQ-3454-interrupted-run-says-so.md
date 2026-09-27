---
id: REQ-3454
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3454

A run that stopped because it was interrupted MUST say so.

The alternative was to exit zero, since nothing failed, which lost because a
script would read a stopped apply as a finished one and go on to the next step,
and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-054` in `docs/spec/cli.md`, its first obligation.
