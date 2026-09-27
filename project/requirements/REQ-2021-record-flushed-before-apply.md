---
id: REQ-2021
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2021

A record MUST be appended and flushed to disk before its operation is applied.

A crash between the append and the effect leaves a journal that replays a no-op,
which is safe; the reverse order leaves an effect with no undo, which is not
(from docs/spec/ops.md, high). The alternative was to apply and then journal,
which lost for that reason (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-021` in `docs/spec/ops.md`.
