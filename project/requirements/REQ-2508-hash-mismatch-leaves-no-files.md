---
id: REQ-2508
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2508

A hash mismatch MUST NOT leave the extracted files in the cache.

The alternative was to warn and continue, which lost because a hash mismatch is
either corruption or tampering, and both want a stop, and the table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-031` in `docs/spec/module.md`, its third obligation.
