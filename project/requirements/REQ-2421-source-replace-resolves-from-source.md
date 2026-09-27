---
id: REQ-2421
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2421

`replace(name, source)` MUST resolve the module from the replacement source.

The alternative was to skip verification for a fork too, which lost because a
remote fork is remote input, and the `replace` says where to fetch and not that
the fork is trusted, and the table names no condition that would reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-021` in `docs/spec/module.md`, its first obligation.
