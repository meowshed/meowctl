---
id: REQ-3013
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3013

Execution order MUST be a topological sort of the dependency graph, with
declaration order as the tie-break between components that have no dependency
between them.

`TopoSort` in `internal/lifecycle/toposort.go` takes the declaration-order map
for exactly this (from docs/spec/engine.md, high). The alternative was sorting
alphabetically, which lost because declaration order is what a user controls,
while alphabetical order is arbitrary and changes on rename (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-013` in `docs/spec/engine.md`.
