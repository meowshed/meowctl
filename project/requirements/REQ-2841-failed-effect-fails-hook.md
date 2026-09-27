---
id: REQ-2841
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2841

A failed effect MUST propagate as a hook failure, which fails the component and
triggers rollback; see REQ-3030.

The alternative was to log and continue, which lost because a component would be
reported as installed when its writes failed (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-041` in `docs/spec/ctx.md`.
