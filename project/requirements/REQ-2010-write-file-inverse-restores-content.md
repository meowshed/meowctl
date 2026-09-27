---
id: REQ-2010
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2010

`WriteFile`'s inverse MUST restore the prior content when the file existed.

`inverseWriteFile` carries exactly this distinction, and the `had_prior` flag is
what separates them (from docs/spec/ops.md, high). The alternative was to always
delete on undo, which lost because undoing a write to a file that existed would
destroy the user's original (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-010` in `docs/spec/ops.md`, its first obligation.
