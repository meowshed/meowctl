---
id: REQ-3505
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-3505

Code below `meowctl-cli` MUST NOT consult the dry-run flag except to construct
an effect implementation or to report the flag to a hook.

The alternative was to pass `dry_run` down, which lost because that is what
`v0.1.0` does and the branch was missed once, and the trade-off record names no
condition that reverses it (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The missed branch shipped as
`fix(apply): dry-run claimed work that the runner skips` (from CLAUDE.md, high).

The engine reads its `Settings::dry_run` in two places: `effects_for` wraps the
executor in a `DryRunExecutor` built for the phase, and `capabilities` copies
the flag into `ctx.dry_run` (from
crates/meowctl-engine/src/runner.rs:382 and :567, high). Both are allowed,
because the first constructs an implementation, which REQ-3034 requires, and
the second fills an attribute the `v0.1.0` surface requires, so a boolean has
to reach `ctx` whatever carries it (from docs/spec/ctx.md:30, high). ADR-0004
records the verdict.

Onboarding narrowed the wording from forbidding `--dry-run` to be passed down as
a boolean, which parity makes impossible to meet literally; the property it
protects is that no code asks whether this is a dry run while it does work
(from CLAUDE.md, high).

Migrated from `R-CLI-011` in `docs/spec/cli.md`, its second obligation.
