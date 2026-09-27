---
id: REQ-2003
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2003

`Op::inverse` MUST be computed before the effect is applied, because it depends
on the state the effect is about to destroy: the prior content of a file, the
prior target of a symlink, whether a directory already existed.

The alternative was to compute the inverse after applying, which lost because
the state it needs has been destroyed by then (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-003` in `docs/spec/ops.md`.
