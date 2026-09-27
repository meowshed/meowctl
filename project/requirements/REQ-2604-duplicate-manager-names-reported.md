---
id: REQ-2604
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2604

Two components exporting the same `pm_name` MUST be reported rather than
silently resolved.

`v0.1.0` overwrites the earlier handler, which makes which one wins depend on
evaluation order (from docs/spec/pm.md, high). The alternative was to overwrite,
as `v0.1.0` does, which lost because which handler wins then depends on graph
order (from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a
configuration is found that legitimately shadows a handler (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The choice is argued at
length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-PM-004` in `docs/spec/pm.md`.
