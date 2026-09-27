---
id: ADR-0007
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3001]
supersedes: []
---

# 0007. Make discovery, the graph, the plan and the report distinct types

## Decision

`Discovered` → `Graph` → `Plan` → `Report` makes the two-pass evaluation a
type-level fact (from crates/meowctl-engine/src/discovery.rs:95,
crates/meowctl-engine/src/graph.rs:28 and crates/meowctl-engine/src/plan.rs:49,
high). The `Registry` of package-manager handlers travels on `Discovered`, and
`Runner::new` takes it, so a `Runner` can't exist before registration and no
hook runs before it (from crates/meowctl-engine/src/discovery.rs and
crates/meowctl-engine/src/runner.rs, high). REQ-3001 states it (from
docs/spec/engine.md, high).

The plan named the stages `Resolved`, `Planned` and `Executed`, and no pull
request records the rename; the property it wanted holds with the shipped
names (from docs/design/0.2.0-rust-rewrite.md §3, high).

Once accepted, a stage can't be entered before the one it depends on has
produced its value.

## Why

In `v0.1.0` "pass one registers PM handlers" is a comment, not a type (from
docs/design/0.2.0-rust-rewrite.md §2 defect #9, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Staged comments: one pass order described in a comment, as `v0.1.0` does | No type per stage | A caller can ignore a comment, which is how the ordering became implicit (from docs/design/0.2.0-rust-rewrite.md §2 and §3, high) |

## What it costs

The source records no cost for this choice (from
docs/design/0.2.0-rust-rewrite.md §3, high). A type per stage means a caller has
to carry each stage's value to the next, which is the point of it (inferred,
low).

## What would reverse it

- The stages collapse into one genuinely atomic step (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- REQ-3001 is structural: the alternative doesn't compile, so no behavioural
  test names it beyond `crates/meowctl-engine/tests/running.rs` (from
  https://github.com/meowshed/meowctl/pull/89, high).

## How I will know it was realised

1. `crates/meowctl-engine/tests/running.rs` holds REQ-3001, and a graph test
   checks handler registration happens before any hook could run (from
   crates/meowctl-engine/tests/running.rs and
   https://github.com/meowshed/meowctl/pull/72, high).

## What this does not settle

- The source recorded no alternative beyond staged comments (from
  docs/design/0.2.0-rust-rewrite.md §3, high).
