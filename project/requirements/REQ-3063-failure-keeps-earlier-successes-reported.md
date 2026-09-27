---
id: REQ-3063
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3063

A component that fails MUST NOT prevent the phase from reporting the components
that already succeeded.

The alternative was reporting the failure alone, which lost because a user
cannot tell what already applied (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-063` in `docs/spec/engine.md`.
