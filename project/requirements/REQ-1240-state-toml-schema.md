---
id: REQ-1240
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1240

`state.toml` MUST hold `schema_version`, a `last_run` table, a
`completed_components` array, and an optional `repo_url`, matching
`internal/state/state.go`.

The alternative was a richer state file, which lost because the file is shared
with `v0.1.0`; revisit it if the Go tree is deleted (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-040` in `docs/spec/config.md`.
