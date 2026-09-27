---
id: REQ-2505
artifact: requirement
topic: module
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2505

`replace(name, source)` MUST verify its integrity normally: a remote fork is not
more trusted than the original.

The alternative was to skip verification for a fork too, which lost because a
remote fork is remote input, and the `replace` says where to fetch and not that
the fork is trusted, and the table names no condition that would reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-021` in `docs/spec/module.md`, its second obligation.
