---
id: REQ-2305
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2305

A Starlark `fail()` call MUST exit with the configuration code.

The alternative was to treat `fail()` as a general error, which lost because
`fail()` is how a component reports a configuration problem, and the table names
no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-042` in `docs/spec/starlark.md`, its second obligation.
