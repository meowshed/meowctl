---
id: REQ-3042
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3042

A non-empty rollback journal at startup MUST be reported as an interrupted
previous run before anything else happens; see REQ-2025.

The alternative was replaying the journal silently, or ignoring it, which lost
because either surprises the user: one changes their system without asking, and
the other loses the undo (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-042` in `docs/spec/engine.md`.
