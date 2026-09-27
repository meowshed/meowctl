---
id: REQ-2026
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2026

An `Op` MUST NOT be journaled during a dry run.

The `DryRunFs` makes the effects no-ops, and a journal of things that did not
happen would replay into damage (from docs/spec/ops.md, high). The alternative
was to journal the dry run, which lost for that reason (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The choice is argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-026` in `docs/spec/ops.md`, its first obligation.
