---
id: REQ-2301
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2301

`unpkg` and `uppkg` MUST take the same shape as `pkg` for removal and update.

The alternative was a separate builtin per manager, which lost because handlers
are components and not code, so the manager is an argument, and the table names
no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-003` in `docs/spec/starlark.md`, its second obligation.
