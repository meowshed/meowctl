---
id: REQ-3531
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3531

`self-update` MUST report being up to date and change nothing when the tag and
the running version match.

The alternative was to update to whatever the tag says, which lost because
replacing a binary with itself is work, risk and a false claim to have done
something, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-072` in `docs/spec/cli.md`, its third obligation.
