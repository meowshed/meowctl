---
id: REQ-2441
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2441

A cached module whose per-file hashes match what was recorded at extraction MUST
be used without a network request.

The alternative was to re-fetch always, which lost because every shell spawn
that triggers resolution would hit the network, and the table names no condition
that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-041` in `docs/spec/module.md`.
