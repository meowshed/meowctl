---
id: REQ-3422
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3422

A dry run MUST render the `Plan`.

The alternative was to print a summary, which lost because the plan is the whole
point of a dry run, and the trade-off record names no condition that reverses it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-022` in `docs/spec/cli.md`, its first obligation.
