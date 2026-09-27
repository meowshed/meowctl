---
id: REQ-1011
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1011

`PhaseSet` MUST have the five variants `install`, `update`, `upgrade`,
`uninstall`, and `verify`, each yielding its phases in the order
`internal/lifecycle/runner.go` gives: `install` yields `install_check`,
`install`, `install_configure`; `update` yields `update`; `upgrade` yields
`upgrade_check`, `upgrade`, `upgrade_configure`; `uninstall` yields
`uninstall_check`, `uninstall`, `uninstall_cleanup`; `verify` yields `verify`.

The alternative was letting each command list its own phases, which lost because
the mapping is fixed and duplicating it is how a phase gets dropped from one
command; revisit it if a command needs a phase set of its own (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-011` in `docs/spec/common.md`.
