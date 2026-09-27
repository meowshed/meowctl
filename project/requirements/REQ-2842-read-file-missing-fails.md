---
id: REQ-2842
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2842

`read_file` on a missing file MUST fail.

The two are how a component tests and then reads, and conflating them turns a
test into an error (from docs/spec/ctx.md, high). The alternative was to make
both fail, or both return false, which lost for that reason (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-042` in `docs/spec/ctx.md`, its first obligation.
