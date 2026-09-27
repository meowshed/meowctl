---
id: REQ-2031
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2031

An inverse whose target no longer exists MUST succeed rather than fail.

A user who deleted the file meowctl created has already achieved what the
inverse wanted (from docs/spec/ops.md, high). The alternative was to fail when
the target is gone, which lost for that reason (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-031` in `docs/spec/ops.md`.
