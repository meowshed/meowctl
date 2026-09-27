---
id: REQ-2832
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2832

The restriction MUST be enforced by the value the hook receives, not by a check
inside each method.

The alternative was to check inside each method, which lost because that is
twenty-four checks, each a place to forget one (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-032` in `docs/spec/ctx.md`.
