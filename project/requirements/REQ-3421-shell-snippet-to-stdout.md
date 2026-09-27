---
id: REQ-3421
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3421

`shell <shell>` MUST emit its integration snippet to stdout, because the output
is evaluated by the shell.

This is the one command whose stdout is a machine interface (from
docs/spec/cli.md, high). The alternative was to route the snippet through the
sink's formatting, which lost because the shell evaluates this output and
decoration is a syntax error, and the trade-off record names no condition that
reverses it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-021` in `docs/spec/cli.md`, its first obligation.
