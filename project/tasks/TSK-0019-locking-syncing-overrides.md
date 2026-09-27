---
id: TSK-0019
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2420, REQ-2421, REQ-2422, REQ-2450, REQ-2451, REQ-2452, REQ-2453, REQ-2464, REQ-2504, REQ-2505, REQ-2513]
issue:
---

# Locking, syncing, and overrides

This task records issue 21 of the 0.2.0 execution plan, which the plan marks
done, and closes 11 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a manifest and a scripted registry, when a sync runs, then the lock is
   byte-identical to `v0.1.0`'s outside `meta` (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-module/tests/fetching.rs`,
   `crates/meowctl-module/tests/syncing.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Resolve a manifest into a lock, sync it, and apply `replace()` overrides (from
docs/design/0.2.0-execution-plan.md, medium).

The corpus cannot check this, because on purpose it runs no command that reaches
a registry. The oracle is a fixture written by `v0.1.0`'s own `SyncModules`
against a scripted registry, with the tarballs it hashed checked in beside it
(from docs/design/0.2.0-execution-plan.md, high). The comparison excludes
`meta`, the one table the two binaries are meant to disagree about (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0017, issue 19 in the plan: syncing fetches and verifies modules (inferred
  from docs/design/0.2.0-execution-plan.md, high).
- TSK-0018, issue 20 in the plan: syncing selects versions (inferred from
  docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 21 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #68, "Resolve a manifest into a lock, and prove it against v0.1.0's
own", carried the work, matched on "Issue 21" in its body:
https://github.com/meowshed/meowctl/pull/68 (from
https://github.com/meowshed/meowctl/pull/68, high).

## Left alone

The commands. `meowctl dep sync`, `dep add`, `dep upgrade` and the rest are
issue 32; this is the resolution they call (from
https://github.com/meowshed/meowctl/pull/68, high).
