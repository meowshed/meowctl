---
id: REQ-2460
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2460

A registry index that cannot be fetched MUST fail with the module code.

A module that can't be fetched is the failure the module exit code exists for,
so a script can tell it from a configuration mistake; see [REQ-1030] (from
docs/spec/common.md, high).

Migrated from `R-MODULE-060` in `docs/spec/module.md`, its first obligation.
