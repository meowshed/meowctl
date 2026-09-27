---
id: REQ-1230
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1230

`installed.lock` MUST record each installed component with the fingerprint of
the module it was installed from, under schema version 2.

The alternative was recording names only, which lost because that is schema 1,
and a module bump couldn't be detected, and the record expects nothing to
reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-030` in `docs/spec/config.md`.
