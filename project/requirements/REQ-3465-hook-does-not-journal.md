---
id: REQ-3465
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3465

`hook` MUST NOT journal what a hook does.

A `shell` hook that writes is misusing the phase, and a rollback on a shell
spawn would undo the previous one (from docs/spec/cli.md, high). The alternative
was to journal a runtime hook like any other phase, which lost for that reason
and because the phase cannot write anyway; revisit if a runtime hook gains a
mutating method (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-065` in `docs/spec/cli.md`, its first obligation.
