---
id: REQ-3460
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3460

`hook` MUST accept `shell` and `login` and no other phase.

`hook <phase>` is what a shell runs on every spawn, through the snippet
`meowctl shell <shell>` emits (from docs/spec/cli.md, high). The alternative was
to accept every phase name, which lost because `hook install` would run an
install from a shell prompt, and the trade-off record names no condition that
reverses it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-060` in `docs/spec/cli.md`, its first obligation.
