---
id: TSK-0028
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2603, REQ-2615, REQ-2621, REQ-2703, REQ-3030, REQ-3031, REQ-3032, REQ-3033, REQ-3034, REQ-3051, REQ-3060, REQ-3061, REQ-3062, REQ-3063, REQ-3108, REQ-3109, REQ-3110, REQ-3111, REQ-3117, REQ-3118, REQ-3120, REQ-3121]
issue:
---

# Phase execution and rollback

This task records issue 30 of the 0.2.0 execution plan, which the plan marks
done, and closes 23 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a phase set in which one hook fails, when it runs, then the run stops
   at the failure and rolls back the journal (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-engine/tests/graph.rs`,
   `crates/meowctl-engine/tests/running.rs`,
   `crates/meowctl-engine/tests/sentinel.rs`,
   `crates/meowctl-exec/tests/execution.rs`,
   `crates/meowctl-tui/tests/rendering.rs`, `tests/interrupt.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Run a plan phase by phase and roll a failed phase set back (from
docs/design/0.2.0-execution-plan.md, medium). The issue is large on purpose
(from docs/design/0.2.0-execution-plan.md, high).

It also closes the three `meowctl-pm` requirements about when a handler is
called, while issue 22 owns what is called (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0012, issue 13 in the plan: rollback replays the journal (inferred from
  docs/design/0.2.0-execution-plan.md, high).
- TSK-0021, issue 23 in the plan: hooks run with `ctx` (inferred from
  docs/design/0.2.0-execution-plan.md, high).
- TSK-0027, issue 29 in the plan: execution takes the plan (from
  https://github.com/meowshed/meowctl/pull/73, high).

## Evidence

The plan marks issue 30 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #74, "feat(engine): run a plan, dispatch packages, and undo a
failure", carried the work, matched on "Issue 30" in its body:
https://github.com/meowshed/meowctl/pull/74 (from
https://github.com/meowshed/meowctl/pull/74, high).

## Left alone

The sentinel and staleness, which are issue 31 (from
https://github.com/meowshed/meowctl/pull/74, high).
