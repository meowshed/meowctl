---
id: ADR-0035
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3030]
supersedes: []
---

# 0035. Stop a phase at the first component that fails

## Decision

A phase runs its hook for every component in order and stops at the first
component that fails (from docs/spec/engine.md, high). REQ-3030 states it,
reversing what the specification first asked for (from
docs/design/0.2.0-decisions.md §3.19, high).

Once accepted, a failure is reported once, by the component that failed.
Independent failures later in the phase stay unknown until the next run (from
docs/design/0.2.0-decisions.md §3.19, high).

## Why

Components in a phase aren't independent: the graph orders them, and a component
that fails is usually one everything after it needs (from
docs/design/0.2.0-decisions.md §3.19, high). `RunPhase` collected failures until
`fix(lifecycle): fail-fast on component failure and trigger rollback` changed
it, after an apply produced fifty errors saying
`mise: executable file not found` when mise itself was the one that failed (from
docs/design/0.2.0-decisions.md §3.19 and
https://github.com/meowshed/meowctl/pull/74, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Collect every failure in a phase and report them together | Tells a user about independent failures in one run | It reports every dependant of the failed component as a failure too (from docs/design/0.2.0-decisions.md §3.19, high) |

## What it costs

A user learns of one failure per run and the rest stay unknown (from
docs/design/0.2.0-decisions.md §3.19, high).

## What would reverse it

- A phase appears whose components genuinely don't depend on each other (from
  docs/design/0.2.0-decisions.md §3.19, high).

## Consequences

- A failed phase set triggers rollback unless the caller disabled it (from
  docs/spec/engine.md, high).

## How I will know it was realised

1. `crates/meowctl-engine/tests/running.rs` holds REQ-3030 and covers the two
   stopping rules (from crates/meowctl-engine/tests/running.rs and
   https://github.com/meowshed/meowctl/pull/74, high).

## What this does not settle

- The source recorded no alternative beyond collecting failures (from
  docs/design/0.2.0-decisions.md §3.19, high).
