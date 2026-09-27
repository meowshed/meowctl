---
id: REQ-1612
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1612

A test executor MUST be able to answer a scripted set of commands.

`v0.1.0` offers `RunFunc` for exactly this, and it is the only injection point
the Go tree has (from docs/spec/exec.md, high). The alternative was integration
tests with real commands, which lost because every engine test would need `brew`
installed (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-EXEC-012` in `docs/spec/exec.md`, its first obligation.
