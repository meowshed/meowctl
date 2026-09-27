---
id: REQ-3103
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3103

A component reached through transitive expansion and not declared by the
configuration MUST be marked as such, because a `meowctl remove` of one tool
must not run the uninstall hook of the package manager it was reached through;
see REQ-3032.

The alternative was requiring every component to be declared, which lost because
an aggregate module would then need its whole tree copied into `init.star` (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-011` in `docs/spec/engine.md`, its second obligation.
