---
id: REQ-2620
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2620

`query_pm(manager)` MUST call the handler's `interrogate(ctx)` and return its
result to the caller.

It runs during evaluation, on its own evaluation context, which is what
`builtinQueryPM` does with a separate thread (from docs/spec/pm.md, high). The
alternative was to evaluate `interrogate` in the caller's evaluation, which lost
because it would see the caller's accumulator (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-PM-020` in `docs/spec/pm.md`.
