---
id: REQ-2120
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2120

A journal record that cannot be parsed MUST be reported and skipped, because the
alternative is a corrupt line stranding every earlier operation.

The alternative was to abort on a bad record, which lost because one corrupt
line strands every earlier operation (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-030` in `docs/spec/ops.md`, its second obligation.
