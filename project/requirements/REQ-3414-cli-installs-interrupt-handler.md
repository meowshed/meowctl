---
id: REQ-3414
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3414

The binary MUST install a handler for interruption and suspension that finishes
the sink before the process stops, so the cursor is restored; see [REQ-3223].

The handler belongs in the binary because a signal arrives at the process, and a
sink that installed one would be a library taking a process-wide resource (from
docs/spec/cli.md, high). The alternative was to let the sink install its own
handler, which lost because a library taking a process-wide resource surprises
every other caller and two sinks would fight over it, and the trade-off record
names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-014` in `docs/spec/cli.md`.
