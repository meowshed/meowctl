---
id: REQ-3120
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3120

A rollback that partially succeeds MUST record the `partial` outcome.

The alternative was reporting success or failure, which lost because `partial`
is the case that needs manual attention (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-061` in `docs/spec/engine.md`, its second obligation.
