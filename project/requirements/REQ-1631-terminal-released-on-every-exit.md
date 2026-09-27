---
id: REQ-1631
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1631

`TerminalReleased` MUST be emitted even when the process fails or the run is
interrupted.

A renderer that never reclaims the terminal leaves the user without a cursor
(from docs/spec/exec.md, high). The alternative was to release the terminal on
the success path only, which lost because a failed interactive hook leaves the
user with no cursor (from docs/design/0.2.0-requirement-tradeoffs.md, high). The
trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-EXEC-031` in `docs/spec/exec.md`.
