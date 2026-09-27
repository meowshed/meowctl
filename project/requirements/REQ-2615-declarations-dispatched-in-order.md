---
id: REQ-2615
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2615

Declarations MUST be dispatched in declaration order within a component.

The alternative was to batch by manager, which lost because a package that
depends on a repository added by an earlier declaration would run first (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if batching is
measured to matter and ordering is preserved within it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-PM-015` in `docs/spec/pm.md`, its first obligation.
