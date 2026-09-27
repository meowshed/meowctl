---
id: REQ-3521
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3521

`hook` MUST write what `ctx.emit` contributed to stdout and nothing else.

A decorated line would be evaluated by the shell (from docs/spec/cli.md, high).
The alternative was to render the run as any other command does, which lost
because a glyph or an indent on stdout is a command the shell tries to run, and
the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-061` in `docs/spec/cli.md`, its second obligation.
