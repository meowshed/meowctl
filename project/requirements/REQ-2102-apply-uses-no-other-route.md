---
id: REQ-2102
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2102

`Op::apply` MUST NOT touch the filesystem by any other route.

This is what makes a dry run and an in-memory test possible (from
docs/spec/ops.md, high). The alternative was to let an `Op` use `std::fs`, which
lost because dry run and `MemFs` would both stop working (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-002` in `docs/spec/ops.md`, its second obligation.
