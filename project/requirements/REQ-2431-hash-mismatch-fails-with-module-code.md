---
id: REQ-2431
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2431

A hash mismatch MUST fail with the module code.

The alternative was to warn and continue, which lost because a hash mismatch is
either corruption or tampering, and both want a stop, and the table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The module code is the one
[REQ-1030] gives a failed integrity check (from
crates/meowctl-common/src/error.rs:24-25, medium).

Migrated from `R-MODULE-031` in `docs/spec/module.md`, its first obligation.
