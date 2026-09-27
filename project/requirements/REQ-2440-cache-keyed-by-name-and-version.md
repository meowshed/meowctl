---
id: REQ-2440
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2440

A module MUST be cached under the cache directory keyed by name and by resolved
version, or by commit SHA for a GitHub module; see [REQ-1021].

The alternative was one cache entry per module, which lost because two
configurations on one machine at different versions would fight, and the table
names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-040` in `docs/spec/module.md`.
