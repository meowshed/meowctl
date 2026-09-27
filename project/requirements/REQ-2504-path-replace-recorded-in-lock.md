---
id: REQ-2504
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2504

The lock entry for a `replace(name, path)` MUST record that the module is
replaced and where it points; see [REQ-1220].

The alternative was to verify a local replacement, which lost because a local
checkout is edited constantly, and hashing it means re-locking on every save,
and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-020` in `docs/spec/module.md`, its second obligation.
