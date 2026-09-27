---
id: ADR-0024
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1611, REQ-3034, REQ-1770]
supersedes: []
---

# 0024. Choose whether a dry run runs a command by the phase that issued it

## Decision

`DryRunExecutor` runs a command issued from a read-only phase and runs nothing
issued from any other phase, and the phase is resolved when the executor is
constructed, so nothing below `meowctl-cli` branches on a dry run (from
docs/design/0.2.0-decisions.md §3.5, high). The engine builds one executor per
phase during a dry run, because it's the only place that knows both the flag and
the phase (from https://github.com/meowshed/meowctl/pull/74, high). REQ-1611 and
REQ-3034 state it (from docs/spec/exec.md and docs/spec/engine.md, high).

Once accepted, an `install_check` hook can still interrogate the system in a dry
run, and a skipped command reports success, which REQ-1770 states (from
https://github.com/meowshed/meowctl/pull/58, high). What doesn't work yet: the
binary's dry run stops at the plan and never builds these executors; BUG-0020
records that (from crates/meowctl-cli/src/run/commands.rs:449-455, high).

## Why

Refusing every subprocess makes check hooks useless, because they interrogate
the system (from docs/design/0.2.0-decisions.md §3.5, high). A per-call flag is
new Starlark surface and puts the decision in the component author's hands,
where a mistake is silent (from docs/design/0.2.0-decisions.md §3.5, high).
Deciding by phase reuses the set `v0.1.0` already has in `validCheckPhases`
(from docs/design/0.2.0-decisions.md §3.5, high). A skipped command reports
success because a hook that branches on failure would otherwise plan a repair
that isn't needed (from https://github.com/meowshed/meowctl/pull/58, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Refuse every subprocess in a dry run | Nothing runs at all | Check hooks interrogate the system, so their plan would be empty (from docs/design/0.2.0-decisions.md §3.5, high) |
| A per-call flag | The author says which calls are safe | New Starlark surface, and a mistake is silent (from docs/design/0.2.0-decisions.md §3.5, high) |

## What it costs

The engine carries the dry-run flag to build a per-phase executor, which is a
choice of implementation rather than a method branching on a flag (from
https://github.com/meowshed/meowctl/pull/74, high).

## What would reverse it

- A read-only phase is found to mutate (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- `interrogate` is callable during a dry run (from
  https://github.com/meowshed/meowctl/pull/69, high).
- `validCheckPhases`, which the specification first cited, doesn't exist in
  `v0.1.0`; the read-only set lives in REQ-1012 (from
  https://github.com/meowshed/meowctl/pull/70, high).

## How I will know it was realised

1. `crates/meowctl-exec/tests/execution.rs` holds REQ-1611 and REQ-3034, and a
   runner test gets an answer from `query_pm` in `install_check` and an empty
   list in `install` under `--dry-run` (from those files and
   https://github.com/meowshed/meowctl/pull/74, high).
2. `effects_for` in `crates/meowctl-engine/src/runner.rs` builds a
   `DryRunExecutor` per phase (from crates/meowctl-engine/src/runner.rs, high).

## What this does not settle

- Whether the reasoning in docs/design/0.2.0-decisions.md §3.5 still holds now
  that `validCheckPhases` turned out not to exist (from
  https://github.com/meowshed/meowctl/pull/70, medium).
