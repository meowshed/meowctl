---
id: REQ-3041
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3041

A component MUST be recorded as completed immediately after its hook succeeds,
not at the end of the phase, so an interrupted run resumes where it stopped.

Because each completion is written as it happens, the sentinel needs no flush
when a run is interrupted (from docs/spec/engine.md, high). The alternative was
recording at the end of the phase, which lost because an interrupted run would
redo everything it had already done (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-041` in `docs/spec/engine.md`.
