---
id: BUG-0020
artifact: bug
status: approved
severity: major
violates: REQ-3172
found: 2026-09-27
revised: 2026-09-27
issue:
---

# `apply --dry-run` runs no hook, so the per-phase dry-run executor is dead code in the binary

The binary never sets the engine's `Settings::dry_run` to true, so under
`--dry-run` no read-only phase runs, `DryRunExecutor` is never built, and
`ctx.dry_run` is `False` in every hook that does run (from
crates/meowctl-cli/src/run/commands.rs:449-455, :481 and :598, high).

## Reproduction

Seen at revision `7e8cabe`. The research read the code and didn't run the binary
against a configuration with an `install_check` hook:

1. `rg -n 'dry_run' crates/meowctl-cli/src` shows `apply` returning after it
   emits `PlanComputed` when `session.dry_run` is set, and both `Settings`
   values it builds passing `dry_run: false` (from
   crates/meowctl-cli/src/run/commands.rs:449-455, :481 and :598, high).
2. `crates/meowctl-cli/src/run.rs:246-261` selects `DryRunFs` for a dry run and
   always passes `RealExecutor` (from that file, high).
3. `rg -n 'dry_run: true' crates` finds one hit,
   `crates/meowctl-engine/tests/running.rs:527` (from that search, high).

## What the system does

`meowctl apply --dry-run` renders the plan and stops. No hook runs, so an
`install_check` hook never interrogates the system, `effects_for`'s dry-run
branch in `crates/meowctl-engine/src/runner.rs:381-392` is reached only from
engine tests, and `CHANGELOG.md` says a dry run "predicts exactly what a real
run would do" through a `FileSystem` and an `Executor` that can't write (from
those files, high).

## What it should do, and why

No requirement in force settles it, so `violates` is omitted. REQ-3101 and
REQ-3102 say a dry run renders the plan and doesn't execute it, and the binary
meets them. REQ-3034, REQ-1611 and REQ-2621 describe a dry run in which
read-only phases run through a per-phase executor and `interrogate` answers, and
in the binary they are reachable only from engine tests (from those
requirements, high). Either the dry run runs the read-only phases through the
engine with `dry_run: true`, or REQ-3034, REQ-1611 and REQ-2621 state that they
apply only when a caller asks the engine for a dry run, and the changelog stops
claiming the prediction (from the research on decision 1, high). The owner
chooses; that is a gap in the requirements and routes to the requirements step.

## Triage

It enters at `meowctl-cli`, which never hands the engine a dry run (from
crates/meowctl-cli/src/run/commands.rs:447-454, high). Major, because a dry run
reports success for a hook that will fail, the silent false success ADR-0023
calls worse than no dry run, and because `upgrade --dry-run` and
`verify --dry-run` ran hooks under `v0.1.0` and run none now (from
`git show v0.1.0:internal/cli/lifecycle.go` lines 849-940, high). REQ-3172,
added to settle whether a dry run runs hooks, is the requirement it violates.

## Closed by

Open. A test that drives the binary under `--dry-run` would show an
`install_check` hook running with `ctx.dry_run` true, or the requirements would
be reworded to match the plan-only dry run.
