---
id: REQ-3033
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3033

A component whose hook is absent MUST be recorded as completed, not skipped.

Absence means the component has nothing to do in this phase, which is a success;
see REQ-2231 (from docs/spec/engine.md, high). The alternative was recording it
as a skip, which lost because a component with nothing to do in a phase has
succeeded at it, and calling it a skip makes every plan noisy (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-033` in `docs/spec/engine.md`, its first obligation.
