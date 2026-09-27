---
id: REQ-3523
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3523

`status` MUST say that the hook run failed and where to look.

The alternative was to say nothing and let the user find the file, which lost
because a shell silently missing its integration is exactly what nobody thinks
to look for, and the trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-064` in `docs/spec/cli.md`, its second obligation.
