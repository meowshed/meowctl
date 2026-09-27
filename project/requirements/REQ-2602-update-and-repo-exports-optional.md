---
id: REQ-2602
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2602

`update_pkg` and `add_repo` MUST be optional.

The alternative was to require all six exports, which lost because most handlers
never needed an update path (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-PM-002` in `docs/spec/pm.md`.
