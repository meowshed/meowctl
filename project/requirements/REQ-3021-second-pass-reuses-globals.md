---
id: REQ-3021
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3021

The second pass MUST call hooks in graph order, reusing the globals from the
first rather than re-evaluating.

The alternative was re-evaluating per phase, which lost because that means
thirteen evaluations of every component, and a component with side effects at
module scope would repeat them (from docs/design/0.2.0-requirement-tradeoffs.md,
high). Revisit it if a component legitimately needs per-phase evaluation (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-021` in `docs/spec/engine.md`.
