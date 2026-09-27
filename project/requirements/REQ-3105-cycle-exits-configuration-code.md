---
id: REQ-3105
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3105

A cycle MUST exit with the configuration code.

`TopoSort` reports only that there is a cycle, which leaves a user with a
hundred components and no way to find the two that point at each other (from
docs/spec/engine.md, high). The alternative was breaking the cycle arbitrarily,
which lost because that gives a silently chosen order that changes between runs
(from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table
names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-015` in `docs/spec/engine.md`, its second obligation.
