---
id: REQ-2306
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2306

A file that does not parse MUST NOT partially apply what came before it.

The alternative was to apply what parsed, which lost because half a
configuration applied is worse than none, and the table names no condition that
would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-050` in `docs/spec/starlark.md`, its second obligation.
