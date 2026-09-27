---
id: REQ-3519
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3519

A run that stopped because it was interrupted MUST exit with the general code.

Not zero, because the command did not do what was asked, and a script that
treats an interrupted `apply` as a successful one goes on to the next step (from
docs/spec/cli.md, high). Not a code of its own either, because [REQ-1030] fixes
the four codes `v0.1.0` defines and says "and no others" (from docs/spec/cli.md,
high). The alternative was to exit zero, since nothing failed, which lost
because a script would read a stopped apply as a finished one and go on to the
next step, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-054` in `docs/spec/cli.md`, its second obligation.
