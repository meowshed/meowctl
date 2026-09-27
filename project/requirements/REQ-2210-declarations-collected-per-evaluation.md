---
id: REQ-2210
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2210

Declarations MUST be collected per evaluation, not in a global.

`v0.1.0` reaches the accumulator through thread-local state, and the equivalent
must be per-evaluation here, so a second evaluation does not see the first one's
declarations (from docs/spec/starlark.md, high). The alternative was a global
accumulator, which lost because two evaluations in one process would see each
other's declarations, and the table names no condition that would reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-010` in `docs/spec/starlark.md`.
