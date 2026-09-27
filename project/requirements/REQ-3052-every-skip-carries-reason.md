---
id: REQ-3052
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3052

Every skip MUST carry its reason: already completed, filtered out, or platform
mismatch.

An absent hook is not a skip; see REQ-3033 (from docs/spec/engine.md, high). A
skip a user cannot explain is what made `v0.1.0`'s dry run misleading (from
docs/spec/engine.md, high). `v0.1.0` carries no reason on a skip, and the plan
needs one in order to be worth printing (from docs/spec/engine.md, high). The
alternative was reporting a bare count, which lost because "120 components
skipped" is not actionable (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-052` in `docs/spec/engine.md`.
