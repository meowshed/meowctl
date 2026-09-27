---
id: REQ-2614
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2614

Dispatch MUST pass the calling component's `ctx`, so the handler's effects are
journaled against the component that asked for the package rather than against
the handler.

The alternative was to pass the handler's own `ctx`, which lost because effects
would be journaled against the handler, so rollback would attribute them wrongly
(from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table
names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-PM-014` in `docs/spec/pm.md`.
