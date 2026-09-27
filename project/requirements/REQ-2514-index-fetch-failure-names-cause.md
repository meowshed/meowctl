---
id: REQ-2514
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2514

A registry index that cannot be fetched MUST say whether the failure was the
network, the status code, or the parse.

The alternative was one error for every index failure, which lost because the
user cannot tell a network outage from a bad registry URL, and the table names
no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-060` in `docs/spec/module.md`, its second obligation.
