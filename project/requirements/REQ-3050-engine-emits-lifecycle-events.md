---
id: REQ-3050
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3050

The engine MUST emit `PlanComputed` before execution, `PhaseStarted` and
`PhaseFinished` around each phase, and `ComponentStarted`, `ComponentSkipped` or
`ComponentFinished` around each component.

The alternative was emitting at a coarser grain, which lost because a sink could
not show progress per component (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-050` in `docs/spec/engine.md`.
