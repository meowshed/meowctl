---
id: REQ-3011
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3011

A component's dependencies MUST be expanded transitively, so declaring an
aggregate brings in what it needs.

The alternative was requiring every component to be declared, which lost because
an aggregate module would then need its whole tree copied into `init.star` (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-011` in `docs/spec/engine.md`, its first obligation.
