---
id: REQ-3002
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3002

`Plan` MUST be a value computed from the configuration, the lock state, and the
sentinel, without performing an effect.

The alternative was computing the plan while executing it, which lost because a
dry run could not print what a real run would do without doing it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-002` in `docs/spec/engine.md`, its first obligation.
