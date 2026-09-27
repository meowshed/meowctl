---
id: REQ-3031
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3031

A phase set MUST run its phases in the order REQ-1011 gives.

The alternative was continuing past a failed phase, which lost because `install`
would fail and then `install_configure` would run against a half-installed
system (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-031` in `docs/spec/engine.md`, its first obligation.
