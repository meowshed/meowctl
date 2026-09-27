---
id: REQ-2420
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2420

`replace(name, path)` MUST serve every file for that module from the local
directory, skipping fetch, cache, and integrity verification.

The alternative was to verify a local replacement, which lost because a local
checkout is edited constantly, and hashing it means re-locking on every save,
and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-020` in `docs/spec/module.md`, its first obligation.
