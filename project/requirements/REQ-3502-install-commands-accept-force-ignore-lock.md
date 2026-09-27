---
id: REQ-3502
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3502

`apply`, `add` and `upgrade` MUST additionally accept `--force` and
`--ignore-lock`, matching `v0.1.0`.

These are the three commands that call `addInstallFlags` in
`internal/cli/lifecycle.go` (from docs/spec/cli.md, high). An earlier draft put
`--dry-run` on each mutating command and the other flags on `apply` alone, and
the implementation followed it: five commands shipped without `--no-rollback`,
two without `--force`, and `verify` without `--dry-run` (from docs/spec/cli.md,
high). The alternative was a global `--dry-run`, which lost because a dry run
means nothing for `status` or `version`, and the trade-off record names no
condition that reverses it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CLI-005` in `docs/spec/cli.md`, its second obligation.
