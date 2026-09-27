---
id: REQ-2005
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2005

An `Op` that finds the system already in its target state MUST still produce a
correct inverse.

Re-linking an already-correct symlink must not journal a removal that would
delete the user's working configuration (from docs/spec/ops.md, high). The
alternative was to skip journaling a no-op, which lost for that reason (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-005` in `docs/spec/ops.md`.
