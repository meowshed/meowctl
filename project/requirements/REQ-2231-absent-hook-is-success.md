---
id: REQ-2231
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2231

A hook that is absent MUST be a success, not an error.

Hooks are optional, and most components define two or three of the thirteen
(from docs/spec/starlark.md, high). The alternative was to require every hook,
which lost because most components define three of thirteen, and the table names
no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-031` in `docs/spec/starlark.md`.
