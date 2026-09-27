---
id: REQ-2422
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2422

An override in `deps.local.mod` MUST win over the same module in `deps.mod`,
because the local file is how one machine differs from the rest.

The alternative was to merge the two, which lost because a machine-local
override exists to differ from the committed manifest; revisit if parity is
abandoned (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-022` in `docs/spec/module.md`.
