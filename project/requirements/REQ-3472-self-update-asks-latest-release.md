---
id: REQ-3472
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3472

`self-update` MUST ask for the latest release.

The alternative was to update to whatever the tag says, which lost because
replacing a binary with itself is work, risk and a false claim to have done
something, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-072` in `docs/spec/cli.md`, its first obligation.
