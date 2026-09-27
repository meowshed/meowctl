---
id: REQ-2222
artifact: requirement
topic: starlark
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2222

A module file MUST be integrity-checked before it is evaluated; see [REQ-2430].

Evaluating first and checking afterwards means the code already ran (from
docs/spec/starlark.md, high). The alternative was to verify after evaluating,
which lost because the code has already run, and the table names no condition
that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-022` in `docs/spec/starlark.md`.
