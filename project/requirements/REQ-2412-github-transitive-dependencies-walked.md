---
id: REQ-2412
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2412

A GitHub module's transitive dependencies MUST be read from its own manifest and
walked, so an aggregate module brings its dependencies with it.

The alternative was to ignore the transitive dependencies of GitHub modules,
which lost because an aggregate module would resolve to itself and break, and
the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-012` in `docs/spec/module.md`.
