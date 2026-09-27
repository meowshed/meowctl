---
id: REQ-1307
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1307

A `packages` entry is keyed by manager, then by package, and MUST carry
`requested`, `installed`, and optionally `note`.

The alternative was a cleaner schema, which lost because the schema is shared
with `v0.1.0`, and a key rename is a broken lock for anyone with both binaries;
revisit it if the Go tree is deleted and a migration ships (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-020` in `docs/spec/config.md`, its fourth obligation.
