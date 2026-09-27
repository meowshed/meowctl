---
id: REQ-3044
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3044

Staleness MUST be computed from the fingerprint in REQ-1233, so a GitHub module
re-synced to a new commit invalidates even though no version changed.

The alternative was using the version, which lost because a GitHub module
re-synced to a new commit keeps its version (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-044` in `docs/spec/engine.md`.
