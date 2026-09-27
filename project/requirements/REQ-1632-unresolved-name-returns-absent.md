---
id: REQ-1632
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1632

Resolving a name on `PATH` MUST return an absent result rather than an error
when nothing matches, because `ctx.which` is how a component asks whether a tool
is present; see REQ-2821.

The alternative was to raise an error when nothing is on `PATH`, which lost
because `ctx.which` is how a component asks whether a tool exists (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-EXEC-032` in `docs/spec/exec.md`.
