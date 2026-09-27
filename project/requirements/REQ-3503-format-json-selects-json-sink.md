---
id: REQ-3503
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3503

`--format json` MUST select `JsonSink`; see [REQ-3232].

The alternative was to keep `--json` on two commands, which lost because every
other command would stay unscriptable, and the trade-off record names no
condition that reverses it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CLI-006` in `docs/spec/cli.md`, its second obligation.
