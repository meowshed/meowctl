---
id: REQ-3451
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3451

An interrupt MUST reach the engine, which stops cleanly; see [REQ-3062].

The alternative was to let the signal kill the process, which lost because the
sentinel would be left unflushed and the terminal unrestored, and the trade-off
record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-051` in `docs/spec/cli.md`, its first obligation.
