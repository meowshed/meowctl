---
id: REQ-2603
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2603

Registration MUST happen during the first evaluation pass, before any hook runs,
so that a hook in the first component can declare a package handled by the last.

See REQ-3020 (from docs/spec/pm.md, high). The alternative was to register
lazily on first use, which lost because a hook in the first component could not
declare a package handled by the last (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-PM-003` in `docs/spec/pm.md`.
