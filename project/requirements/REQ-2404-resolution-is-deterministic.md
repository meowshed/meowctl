---
id: REQ-2404
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2404

A resolution MUST be deterministic.

The alternative was to let map-iteration order leak, which lost because it gives
a lock file that differs between machines for the same manifests, and the table
names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-004` in `docs/spec/module.md`, its first obligation.
