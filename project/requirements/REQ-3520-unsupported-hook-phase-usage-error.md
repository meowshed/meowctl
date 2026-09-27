---
id: REQ-3520
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3520

An unsupported phase MUST exit 2 with the usage error, naming the two that work;
see [REQ-1013].

The alternative was to accept every phase name, which lost because
`hook install` would run an install from a shell prompt, and the trade-off
record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-060` in `docs/spec/cli.md`, its second obligation.
