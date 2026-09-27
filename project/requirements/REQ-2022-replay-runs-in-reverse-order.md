---
id: REQ-2022
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2022

Replay MUST apply inverses in reverse sequence order.

The alternative was forward order, which lost because it undoes a `mkdir` before
the writes inside it (from docs/design/0.2.0-requirement-tradeoffs.md, high).
The trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-022` in `docs/spec/ops.md`.
