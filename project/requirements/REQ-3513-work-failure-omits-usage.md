---
id: REQ-3513
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3513

A failure in the work MUST NOT print usage, because a wall of flags on top of a
real error buries it.

The alternative was to always print usage on error, which lost because a wall of
flags buries the real error, and the trade-off record names no condition that
reverses it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-032` in `docs/spec/cli.md`, its second obligation.
