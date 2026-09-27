---
id: REQ-2826
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2826

`prompt(question)` MUST go through the `Interaction` trait, not read stdin
directly; see REQ-3260.

The alternative was to read stdin directly, which lost because a prompt in CI
blocks until the job times out (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The choice is argued at
length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-026` in `docs/spec/ctx.md`.
