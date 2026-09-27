---
id: ADR-0006
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3002, REQ-3003]
supersedes: []
---

# 0006. Compute the apply plan as a value, and execute exactly that value

## Decision

`engine::plan(...) -> Plan` is pure given the config and lock state, and
`engine::execute(plan, &effects)` runs it; `--dry-run` prints the plan instead
of executing it (from docs/design/0.2.0-rust-rewrite.md §3, high). REQ-3002 and
REQ-3003 state it (from docs/spec/engine.md, high).

Once accepted, a dry run and a real run can't disagree about what work there is,
because the skip decision is in one place and both callers read it (from
https://github.com/meowshed/meowctl/pull/73, high).

## Why

It makes defect #4 impossible by construction rather than by discipline (from
docs/design/0.2.0-rust-rewrite.md §3, high).
`fix(apply): dry-run claimed work that the runner skips` closed that defect in
the Go tree only for `apply`: the plan listed every component while `RunPhase`
skipped anything already recorded, so a fully applied configuration reported 120
components to install on every run (from
https://github.com/meowshed/meowctl/pull/73, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Rely on each code path checking the dry-run flag correctly, as `v0.1.0` does | No separate plan type | It holds only by discipline, and the plan and the runner drifted apart (from docs/design/0.2.0-rust-rewrite.md §3 and https://github.com/meowshed/meowctl/pull/73, high) |

## What it costs

`apply --dry-run` predicts which components run, not what their hooks do,
because the dry run stops at the plan and runs no hook (from
crates/meowctl-cli/src/run/commands.rs:447-454, high). The parameter cost of
effects behind traits is ADR-0004's (from docs/design/0.2.0-decisions.md §2,
high).

## What would reverse it

- A skip decision needs a fact only a hook can produce, which the plan can't
  hold because it is computed before any hook runs (inferred, low). The
  trade-off rows for REQ-3002 and REQ-3003 read "Never" (from
  docs/design/0.2.0-requirement-tradeoffs.md:308-309, high).

## Consequences

- A component a guard dropped is in the plan with the guard that dropped it,
  rather than missing from it (from https://github.com/meowshed/meowctl/pull/73,
  high).
- The plan renders one line per component, not one per phase and component (from
  https://github.com/meowshed/meowctl/pull/76, high).
- `Settings::dry_run` is never true from the binary, because both callers build
  the runner with `dry_run: false` and the session always holds `RealExecutor`,
  so `effects_for`'s dry-run executor is reached only from engine tests;
  BUG-0020 records it (from crates/meowctl-cli/src/run/commands.rs:481 and
  :598, crates/meowctl-cli/src/run.rs:261, high).

## How I will know it was realised

1. `crates/meowctl-engine/tests/plan.rs` holds REQ-3002, and
   `crates/meowctl-engine/tests/running.rs` holds REQ-3003 (from those files,
   high).
2. `crates/meowctl-engine/src/plan.rs` defines `pub struct Plan` (from
   crates/meowctl-engine/src/plan.rs, high).

## What this does not settle

- The source recorded no alternative beyond discipline (from
  docs/design/0.2.0-rust-rewrite.md §3, high).
