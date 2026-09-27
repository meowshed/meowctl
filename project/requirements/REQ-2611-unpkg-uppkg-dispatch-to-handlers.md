---
id: REQ-2611
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2611

An `unpkg()` declaration MUST dispatch to `uninstall_pkg`, and a `uppkg()`
declaration to `update_pkg`.

The alternative was to route removal through `install_pkg`, which lost because
the operations differ per manager (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-PM-011` in `docs/spec/pm.md`.
