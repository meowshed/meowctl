---
id: REQ-2253
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2253

A `load()` of a module that cannot be fetched MUST exit with the module code,
not the configuration code; see [REQ-1030].

M0 found that a load the loader refuses surfaces as an ordinary evaluation error
carrying the loader's message (from docs/spec/starlark.md, high). The
alternative was to report a fetch failure as a configuration error, which lost
because exit codes are how scripts tell a broken configuration from a broken
network, and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-053` in `docs/spec/starlark.md`.
