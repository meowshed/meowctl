---
id: REQ-2117
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2117

Replay MUST report which inverses failed.

Stopping at the first failure leaves the rest of the run un-undone, which is
worse than a partial restore (from docs/spec/ops.md, high). The alternative was
to stop at the first failure, which lost for that reason (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-023` in `docs/spec/ops.md`, its second obligation.
