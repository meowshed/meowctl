---
id: REQ-2212
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2212

Declaration order MUST be preserved.

It is the tie-break the component graph uses when two components have no
dependency between them; see REQ-3013 (from docs/spec/starlark.md, high). The
alternative was to sort declarations, which lost because order is the graph's
tie-break, and sorting changes execution order, and the table names no condition
that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-012` in `docs/spec/starlark.md`.
