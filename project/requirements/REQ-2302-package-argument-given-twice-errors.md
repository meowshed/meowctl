---
id: REQ-2302
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2302

Both `manager` and `name` are required, and either may be given positionally or
by keyword; supplying one both ways MUST be an error.

The positional order is `manager` before `name`, and most components write both
arguments by keyword (from docs/spec/starlark.md, high). The alternative was a
separate builtin per manager, which lost because handlers are components and not
code, so the manager is an argument, and the table names no condition that would
reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-003` in `docs/spec/starlark.md`, its third obligation.
