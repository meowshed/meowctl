---
id: REQ-1244
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1244

The rollback outcome MUST be one of the empty string, `ok`, `partial`, or
`failed`, matching the `RolledBack` constants and [REQ-2024].

The alternative was a boolean, which lost because the reason given for
[REQ-2024] applies; revisit it if parity is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-044` in `docs/spec/config.md`.
