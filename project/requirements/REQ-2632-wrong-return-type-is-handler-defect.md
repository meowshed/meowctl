---
id: REQ-2632
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2632

A handler returning a value of an unexpected type MUST be reported as a handler
defect naming the function, rather than being coerced.

The alternative was to coerce the value, which lost because a handler bug
becomes a confusing downstream failure (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-PM-032` in `docs/spec/pm.md`.
