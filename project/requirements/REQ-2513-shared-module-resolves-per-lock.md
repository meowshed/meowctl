---
id: REQ-2513
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2513

A module in both `deps.mod` and `deps.local.mod` MUST resolve independently in
each.

The alternative was one lock for both, which lost because a machine-local module
would land in the committed lock; revisit if parity is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-052` in `docs/spec/module.md`, its second obligation.
