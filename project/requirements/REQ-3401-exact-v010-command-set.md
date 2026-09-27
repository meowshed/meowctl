---
id: REQ-3401
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3401

The command set MUST be exactly what `newRootCmd` in `internal/cli/root.go`
registers: `init`, `apply`, `add`, `remove`, `upgrade`, `verify`, `update`,
`status`, `doctor`, `check`, `hook`, `shell`, `self-update`, `version`, the
`dep` group, and shell completions.

The alternative was to drop a command nobody uses, which lost because a removed
command breaks a script and the project has no usage data (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The record says to revisit
once the Go tree is deleted and 0.3.0 opens the surface; the Go tree is gone, so
only the second half is still open (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-001` in `docs/spec/cli.md`.
