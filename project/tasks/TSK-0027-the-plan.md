---
id: TSK-0027
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3001, REQ-3002, REQ-3003, REQ-3050, REQ-3052, REQ-3100, REQ-3101, REQ-3102]
issue:
---

# The plan

This task records issue 29 of the 0.2.0 execution plan, which the plan marks
done, and closes 8 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a configuration where some components are skipped, when the plan is
   computed, then every skip carries a reason and nothing runs (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-engine/tests/graph.rs`,
   `crates/meowctl-engine/tests/plan.rs`,
   `crates/meowctl-engine/tests/running.rs`,
   `crates/meowctl-tui/tests/rendering.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Compute the `Plan` from the graph, the sentinel and the filter, as a value (from
https://github.com/meowshed/meowctl/pull/73, high). A reason on every skip is
decided in §3.14 of the decisions document and is what makes the plan worth
printing (from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0026, issue 28 in the plan: the plan is computed from the graph (from
  https://github.com/meowshed/meowctl/pull/73, high).
- TSK-0005, issue 6 in the plan: the plan reads `installed.lock` to decide what
  is done (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 29 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #73, "feat(engine): compute what a run will do, as a value",
carried the work, matched on "Issue 29" in its body:
https://github.com/meowshed/meowctl/pull/73 (from
https://github.com/meowshed/meowctl/pull/73, high).

## Left alone

Execution. Issue 30 takes a `Plan` and the effects and runs it (from
https://github.com/meowshed/meowctl/pull/73, high).
