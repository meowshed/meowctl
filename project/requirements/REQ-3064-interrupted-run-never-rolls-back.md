---
id: REQ-3064
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3064

An interrupted run MUST NOT roll back.

The journal is what the next run finds and reports, and undoing work the user
stopped is not what stopping asked for; see REQ-2025 (from docs/spec/engine.md,
high). The alternative was rolling back what the interrupted run had done, which
lost because undoing work the user stopped is not what stopping asked for, and
the journal already tells the next run what is there (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-064` in `docs/spec/engine.md`.
