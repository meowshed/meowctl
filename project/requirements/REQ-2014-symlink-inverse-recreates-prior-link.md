---
id: REQ-2014
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2014

`Symlink`'s inverse MUST re-create the prior symlink when one existed.

The alternative was to always remove the link, which lost because a component
that re-points an existing symlink would leave nothing behind (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-014` in `docs/spec/ops.md`, its first obligation.
