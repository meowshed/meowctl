---
id: REQ-2601
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2601

A component MUST be registered as a handler when it exports all four of
`pm_name`, `install_pkg`, `uninstall_pkg`, and `interrogate`.

The alternative was to register on `pm_name` alone, which lost because a
component with a name and no `install_pkg` accepts declarations and fails at
dispatch (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-PM-001` in `docs/spec/pm.md`, its first obligation.
