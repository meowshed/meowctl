---
id: REQ-2702
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2702

A `repo()` declaration MUST fail naming the manager when the handler exports no
`add_repo`.

The alternative was to ignore `repo()` for a handler without `add_repo`, which
lost because a repository is silently not added, and then packages cannot be
found (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-PM-013` in `docs/spec/pm.md`, its second obligation.
