---
id: REQ-3012
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3012

A component whose platform or distribution guard does not match the current
machine MUST be dropped from the graph.

The alternative was running every component and letting hooks guard, which lost
because every component would need platform checks in every hook (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-012` in `docs/spec/engine.md`.
