---
id: REQ-3412
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3412

Commands MUST be constructed per invocation.

`v0.1.0` holds `var RootCmd = newRootCmd()`, a package-level value built at init
time, which makes the command tree shared mutable state between tests (from
docs/spec/cli.md, high). The alternative, a package-level command tree, lost
because tests share mutable state and depend on order, and the trade-off record
names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-012` in `docs/spec/cli.md`.
