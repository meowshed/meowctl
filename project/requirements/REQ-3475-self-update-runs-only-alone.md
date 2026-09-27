---
id: REQ-3475
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3475

`self-update` MUST NOT run as part of anything else.

It is the one command that changes the tool rather than the machine, and a run
that updated itself halfway through an apply would finish under a binary that
did not start it (from docs/spec/cli.md, high). The alternative was to let
`apply` update the binary it is running under, which lost for that reason, and
the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-075` in `docs/spec/cli.md`.
