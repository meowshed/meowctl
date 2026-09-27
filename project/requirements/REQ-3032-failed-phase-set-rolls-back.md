---
id: REQ-3032
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3032

A failed phase set MUST trigger rollback unless the caller disabled it.

The alternative was leaving the system as it is, which lost because the journal
exists so a failed run does not leave a half-configured machine (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-032` in `docs/spec/engine.md`, its first obligation.
