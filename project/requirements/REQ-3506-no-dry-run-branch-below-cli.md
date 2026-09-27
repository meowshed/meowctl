---
id: REQ-3506
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-3506

Code below `meowctl-cli` MUST NOT branch on the dry-run flag while it performs
an effect; see [REQ-2814].

Choosing an effect implementation from the flag, as the engine does once per
phase, is not such a branch; REQ-3505 names the two uses allowed (from
crates/meowctl-engine/src/runner.rs:382, high). A new `if dry_run` anywhere in
the tree is a review finding (from CLAUDE.md, high). The alternative was to
pass `dry_run` down, which lost because that is what `v0.1.0` does and the
branch was missed once, and the trade-off record names no condition that
reverses it (from docs/design/0.2.0-requirement-tradeoffs.md, high). The
missed branch shipped as
`fix(apply): dry-run claimed work that the runner skips` (from CLAUDE.md,
high).

Migrated from `R-CLI-011` in `docs/spec/cli.md`, its third obligation.

Onboarding reworded the source's "No ... MUST" to MUST NOT, because the record's
check refuses a negated subject, and then narrowed it to branching while an
effect is performed, so the engine's per-phase choice of executor satisfies it.
