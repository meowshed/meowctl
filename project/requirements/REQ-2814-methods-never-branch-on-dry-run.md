---
id: REQ-2814
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2814

A method MUST NOT branch on whether this is a dry run.

The `FileSystem` and `Executor` it was given decide that; see REQ-1411 (from
docs/spec/ctx.md, high). The alternative was to check `dry_run` per method,
which lost because that is `v0.1.0`, and one branch was missed (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The choice is argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-014` in `docs/spec/ctx.md`.
