---
id: REQ-3406
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3406

`--format json` MUST be available on every command.

In `v0.1.0` JSON output is a hand-written special case on `doctor` (from
docs/spec/cli.md, high). The alternative was to keep `--json` on two commands,
which lost because every other command would stay unscriptable, and the
trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-006` in `docs/spec/cli.md`, its first obligation.
