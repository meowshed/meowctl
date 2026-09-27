---
id: REQ-2821
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2821

`which(name)` MUST return the resolved path or a falsey value when nothing
matches; see REQ-1632.

The alternative was to raise when the tool is absent, which lost because `which`
is a question, not an assertion (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-021` in `docs/spec/ctx.md`.
