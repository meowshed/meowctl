---
id: REQ-3404
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3404

`--config` MUST override the config directory resolution in [REQ-1020].

The alternative was to take the config directory from the environment only,
which lost because a flag is what a script uses for a one-off run, and the
trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-004` in `docs/spec/cli.md`, its first obligation.
