---
id: REQ-2803
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2803

`component_dir` MUST be the component's own source directory and `state_dir` its
persistent per-component directory.

A component writes state it wants to survive into the second (from
docs/spec/ctx.md, high). The alternative was one directory for both, which lost
because source is read-only and replaceable, while state persists across module
upgrades (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-003` in `docs/spec/ctx.md`.
