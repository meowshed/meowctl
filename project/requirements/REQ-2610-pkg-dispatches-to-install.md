---
id: REQ-2610
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2610

A `pkg()` declaration MUST dispatch to
`install_pkg(ctx, name, version, **kwargs)` on the handler for its manager.

The alternative was to install directly from the declaration, which lost because
every manager would then be code in the binary (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-PM-010` in `docs/spec/pm.md`.
