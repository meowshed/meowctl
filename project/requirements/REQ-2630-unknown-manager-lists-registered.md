---
id: REQ-2630
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2630

A `pkg()` naming a manager with no registered handler MUST fail naming the
manager and listing the managers that are registered.

A typo in a manager name is the common cause and the list is what makes it
obvious (from docs/spec/pm.md, high). The alternative was to fail with the
manager name only, which lost for that reason (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-PM-030` in `docs/spec/pm.md`.
