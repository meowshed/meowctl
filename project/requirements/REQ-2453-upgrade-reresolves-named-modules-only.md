---
id: REQ-2453
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2453

An upgrade MUST clear the locked version for the named modules and re-resolve
them, leaving the rest of the lock untouched.

The alternative was to re-resolve everything on upgrade, which lost because
`meowctl dep upgrade one-module` would move every other version too, and the
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-053` in `docs/spec/module.md`.
