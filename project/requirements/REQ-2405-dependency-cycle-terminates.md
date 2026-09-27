---
id: REQ-2405
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2405

A cycle in the dependency graph MUST terminate rather than loop.

`v0.1.0` guards this by not re-expanding a module at a version it has already
visited (from docs/spec/module.md, high). The alternative was to trust manifests
to be acyclic, which lost because a cycle in a third-party module hangs the
tool, and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-005` in `docs/spec/module.md`.
