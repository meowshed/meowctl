---
id: REQ-3450
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3450

A command that needs a configuration and is run outside one MUST say so and
suggest `meowctl init`, rather than reporting a missing file.

`apply`, `add`, `remove`, `upgrade`, `verify`, `update` and the `dep`
subcommands that write all need a configuration and refuse without one (from
docs/spec/cli.md, high). The alternative was to report the missing file, which
lost because "no such file: init.star" tells someone who has not run `init`
nothing, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-050` in `docs/spec/cli.md`, its first obligation.
