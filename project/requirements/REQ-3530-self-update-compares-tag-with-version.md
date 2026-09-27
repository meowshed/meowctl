---
id: REQ-3530
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3530

`self-update` MUST compare the tag of the latest release with the running
version.

The alternative was to update to whatever the tag says, which lost because
replacing a binary with itself is work, risk and a false claim to have done
something, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-072` in `docs/spec/cli.md`, its second obligation.
