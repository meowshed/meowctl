---
id: REQ-3415
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3415

`--no-rollback` on `verify` MUST be accepted.

The alternative was to refuse `--no-rollback` on `verify`, which lost because
`v0.1.0` accepts it and a script passing it everywhere would break on one
command, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-015` in `docs/spec/cli.md`, its first obligation.
