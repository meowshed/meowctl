---
id: REQ-3473
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3473

The asset MUST be the one named for this platform.

The alternative was to report a missing asset as a fetch that failed, which lost
because a user on a platform nobody built for would retry a network that is
fine, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-073` in `docs/spec/cli.md`, its first obligation.
