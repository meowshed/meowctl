---
id: TSK-0020
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2205, REQ-2233, REQ-2601, REQ-2602, REQ-2604, REQ-2610, REQ-2611, REQ-2612, REQ-2613, REQ-2614, REQ-2620, REQ-2630, REQ-2631, REQ-2632, REQ-2700, REQ-2701, REQ-2702, REQ-2704]
issue:
---

# The package-manager registry

This task records issue 22 of the 0.2.0 execution plan, which the plan marks
done, and closes 18 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a component declaring a package for a registered manager, when the
   declaration is dispatched, then the manager's install handler is called with
   the caller's `ctx` (inferred from docs/design/0.2.0-execution-plan.md,
   medium). Closed by: the tests that cite its requirements, in
   `crates/meowctl-engine/tests/graph.rs`,
   `crates/meowctl-engine/tests/running.rs`,
   `crates/meowctl-pm/tests/registry.rs`,
   `crates/meowctl-starlark/tests/evaluation.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Register a package manager from a component that exports `pm_name`,
`install_pkg`, `uninstall_pkg` and `interrogate`, and dispatch declarations to
it (from docs/design/0.2.0-execution-plan.md, medium).

`query_pm` lands here. Issue 14 was recorded as closing it and left it out: it
is the one builtin that runs user code during evaluation, and it cannot be
written before there is a registry to ask (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0013, issue 14 in the plan: handlers are functions the evaluator returns
  (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 22 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #69, "Decide which component handles a manager, and what to call on
it", carried the work, matched on "Issue 22" in its body:
https://github.com/meowshed/meowctl/pull/69 (from
https://github.com/meowshed/meowctl/pull/69, high).

## Left alone

When a handler is called. Registration finishing before any hook runs,
declarations dispatching in graph order and `interrogate` running in a dry run
move to issue 30: this crate produces a call and the runner makes it (from
docs/design/0.2.0-execution-plan.md, high).
