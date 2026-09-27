---
id: REQ-2025
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2025

The journal MUST be truncated after a successful run.

The alternative was to leave the journal, which lost because every run would
replay the last one's operations (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-025` in `docs/spec/ops.md`, its first obligation.
