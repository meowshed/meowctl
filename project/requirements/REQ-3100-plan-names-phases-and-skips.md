---
id: REQ-3100
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3100

`Plan` MUST name the phases to run, the components in each, and which are
skipped and why.

The alternative was computing the plan while executing it, which lost because a
dry run could not print what a real run would do without doing it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-002` in `docs/spec/engine.md`, its second obligation.
