---
id: TSK-0026
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1006, REQ-2234, REQ-3010, REQ-3011, REQ-3012, REQ-3013, REQ-3014, REQ-3015, REQ-3016, REQ-3017, REQ-3018, REQ-3020, REQ-3021, REQ-3103, REQ-3104, REQ-3105, REQ-3106, REQ-3107]
issue:
---

# Discovery and the component graph

This task records issue 28 of the 0.2.0 execution plan, which the plan marks
done, and closes 18 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a configuration whose components depend on each other, when discovery
   runs, then the components come out in topological order, ties broken by
   declaration order (inferred from docs/design/0.2.0-execution-plan.md,
   medium). Closed by: the tests that cite its requirements, in
   `crates/meowctl-common/src/id.rs`, `crates/meowctl-engine/tests/graph.rs`,
   `crates/meowctl-engine/tests/plan.rs`,
   `crates/meowctl-starlark/tests/evaluation.rs`, `tests/layouts.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Discover components and order them into a graph (from
docs/design/0.2.0-execution-plan.md, medium). It also closes two requirements
that discovery needs and neither neighbouring issue had a use for (from
docs/design/0.2.0-execution-plan.md, high).

A later fix made a bare name inside a module resolve beside its declarer, and
the plan now gives this issue that requirement too (from
https://github.com/meowshed/meowctl/pull/79, high).

## Depends on

- TSK-0013, issue 14 in the plan: discovery evaluates every component file
  (inferred from docs/design/0.2.0-execution-plan.md, medium).
- TSK-0014, issue 15 in the plan: discovery reads the declarations the
  accumulator collects (inferred from docs/design/0.2.0-execution-plan.md,
  medium).

## Evidence

The plan marks issue 28 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #72, "feat(engine): discover components and order them", carried
the work, matched on "Issue 28" in its body:
https://github.com/meowshed/meowctl/pull/72 (from
https://github.com/meowshed/meowctl/pull/72, high).

Pull request #79, "fix(engine): read a module's bare dependency beside it",
carried the work, names issue 28 and the requirement it added to it:
https://github.com/meowshed/meowctl/pull/79 (from
https://github.com/meowshed/meowctl/pull/79, high).

## Left alone

The plan, phase execution and the sentinel, which are issues 29 to 31. This
issue produces the order and runs nothing (from
https://github.com/meowshed/meowctl/pull/72, high).
