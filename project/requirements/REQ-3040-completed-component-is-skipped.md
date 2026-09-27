---
id: REQ-3040
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3040

A component already recorded as completed for a phase MUST be skipped, unless
the caller forced a re-run.

The alternative was re-running everything, which lost because `apply` on a
configured machine would redo every install (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-040` in `docs/spec/engine.md`.
