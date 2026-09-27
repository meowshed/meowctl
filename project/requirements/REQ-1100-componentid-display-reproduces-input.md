---
id: REQ-1100
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1100

The `Display` of a `ComponentId` MUST reproduce the input exactly.

The alternative was one string type for a component, which lost because the
three syntaxes have different resolution rules and a string can't say which it
holds, and the record says it never reverses, because the three forms are in
every user's `init.star` (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-COMMON-001` in `docs/spec/common.md`, its second obligation.
