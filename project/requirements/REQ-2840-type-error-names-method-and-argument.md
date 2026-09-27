---
id: REQ-2840
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2840

A method called with a wrong argument type MUST name the method, the argument,
and the expected type.

The alternative was to pass the type error through, which lost because it says
"expected string" without the method or the argument (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-040` in `docs/spec/ctx.md`.
