---
id: REQ-1241
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1241

A `state.toml` whose `schema_version` is higher than this build understands MUST
be reported as written by a newer meowctl.

`v0.1.0` ignores the field entirely, which means an older binary silently
rewrites a newer file and loses whatever it didn't understand (from
docs/spec/config.md, high). The alternative was ignoring `schema_version`, as
`v0.1.0` does, which lost because an older binary silently drops what it didn't
understand; revisit it if the two binaries stop sharing a config directory (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-041` in `docs/spec/config.md`, its first obligation.
