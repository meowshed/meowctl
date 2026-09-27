---
id: REQ-3411
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3411

`--dry-run` MUST select `DryRunFs`, and a dry-run `Executor` for each phase that
runs.

The alternative was to pass `dry_run` down, which lost because that is what
`v0.1.0` does and the branch was missed once, and the trade-off record names no
condition that reverses it (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The missed branch shipped as
`fix(apply): dry-run claimed work that the runner skips` (from CLAUDE.md, high).

The command line selects `DryRunFs` and always passes `RealExecutor`, and the
engine builds the dry-run executor per phase, because only the engine knows the
phase; see REQ-3034 (from crates/meowctl-cli/src/run.rs:246-261, high).
Onboarding reworded the source's "and the read-only `Executor`", which claimed
the command line picks the executor. In the shipped binary a dry run runs no
phase, so no dry-run executor is built; BUG-0020 records that (from
crates/meowctl-cli/src/run/commands.rs:449-455, high).

Migrated from `R-CLI-011` in `docs/spec/cli.md`, its first obligation.
