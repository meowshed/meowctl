---
id: ADR-0032
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3052]
supersedes: []
---

# 0032. Require a reason on every skip

## Decision

Every skip carries its reason: already completed, filtered out, or platform
mismatch (from docs/spec/engine.md, high). REQ-3052 states it (from
docs/design/0.2.0-decisions.md §3.14, high).

Once accepted, a dry run says why each component won't run.

## Why

"120 components skipped" is the line that made `v0.1.0`'s dry run useless, and a
user can't act on it (from docs/design/0.2.0-decisions.md §3.14, high). A
component a user expected and can't find is a bug report (from
https://github.com/meowshed/meowctl/pull/73, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Report a skip without a reason, as `v0.1.0` does | No field to thread through the event | A user can't act on an unexplained skip (from docs/design/0.2.0-decisions.md §3.14, high) |

## What it costs

A reason is another field to thread through the event, and the engine has to
distinguish cases it merged (from docs/design/0.2.0-decisions.md §3.14, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.14, high).

## Consequences

- A component a guard dropped is in the plan with the guard that dropped it
  (from https://github.com/meowshed/meowctl/pull/73, high).

## How I will know it was realised

1. `crates/meowctl-engine/tests/plan.rs`, `crates/meowctl-engine/tests/graph.rs`
   and `crates/meowctl-tui/tests/rendering.rs` hold REQ-3052 (from those files,
   high).

## What this does not settle

- The source recorded no alternative beyond a bare skip (from
  docs/design/0.2.0-decisions.md §3.14, high).
