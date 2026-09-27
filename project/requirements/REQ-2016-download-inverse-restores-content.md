---
id: REQ-2016
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2016

`Download`'s inverse MUST restore prior content when the destination existed.

The alternative was to always delete the download, which lost for the reason
REQ-2010 gives: undoing a write to a file that existed would destroy the user's
original (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-016` in `docs/spec/ops.md`, its first obligation.
