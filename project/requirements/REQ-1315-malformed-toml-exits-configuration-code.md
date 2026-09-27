---
id: REQ-1315
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1315

A malformed TOML file MUST exit with the configuration code; see [REQ-1030].

The alternative was reporting the parse error as it is, which lost because TOML
errors name an offset, not a file, and the record expects nothing to reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-060` in `docs/spec/config.md`, its second obligation.
