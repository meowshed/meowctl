---
id: REQ-3003
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3003

Execution MUST take a `Plan` and the effects.

Rendering the plan and never executing it during a dry run is how REQ-1411 and
REQ-2814 add up to a guarantee rather than a convention (from
docs/spec/engine.md, high). The alternative was letting execution recompute the
plan, which lost because it gives two code paths for one decision, and they
drift (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-003` in `docs/spec/engine.md`, its first obligation.
