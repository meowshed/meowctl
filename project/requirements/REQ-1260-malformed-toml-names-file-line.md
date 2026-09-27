---
id: REQ-1260
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1260

A malformed TOML file MUST produce an error naming the file and the line.

The alternative was reporting the parse error as it is, which lost because TOML
errors name an offset, not a file, and the record expects nothing to reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-060` in `docs/spec/config.md`, its first obligation.
