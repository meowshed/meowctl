---
id: REQ-2403
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2403

A dependency declaring a version that is not valid semver MUST fail resolution
naming the module and the version, rather than being skipped or treated as
`none`.

The alternative was to skip an unparseable version, which lost because a typo
silently drops a dependency, and the table names no condition that would reverse
it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-003` in `docs/spec/module.md`.
