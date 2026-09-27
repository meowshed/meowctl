---
id: REQ-3516
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3516

A command that only reports MUST NOT refuse.

`status` in an empty directory says no runs are recorded and exits zero, and
`dep list` prints an empty list; both are answers to the question asked (from
docs/spec/cli.md, high). The alternative was to report the missing file, which
lost because "no such file: init.star" tells someone who has not run `init`
nothing, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-050` in `docs/spec/cli.md`, its third obligation.
