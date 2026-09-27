---
id: REQ-1224
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1224

`deps.local.lock` MUST have the same schema as `deps.lock`.

The alternative was one lock with a scope column, which lost because `v0.1.0`
keeps them separate and `.gitignore`s the local one; revisit it if parity is
abandoned (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-024` in `docs/spec/config.md`, its first obligation.
