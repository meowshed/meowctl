---
id: TSK-0016
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2230, REQ-2231, REQ-2232, REQ-2240, REQ-2241, REQ-2242, REQ-2250, REQ-2251, REQ-2252, REQ-2305, REQ-2306, REQ-2307]
issue: 114
projected: faa975536266
---

# Calling hooks and diagnostics

This task records issue 17 of the 0.2.0 execution plan, which the plan marks
done, and closes 12 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a component that fails inside a module it loaded, when it is evaluated,
   then the error names the source position and the chain of `load()` calls
   (inferred from docs/design/0.2.0-execution-plan.md, medium). Closed by: the
   tests that cite its requirements, in
   `crates/meowctl-starlark/tests/evaluation.rs`,
   `crates/meowctl-starlark/tests/stdlib.rs`,
   `crates/meowctl-tui/tests/rendering.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Call a hook with `ctx`, and report evaluation errors with their source position
and load chain (from docs/design/0.2.0-execution-plan.md, medium). M0 found the
library already carries the source position; the work is carrying the load chain
(from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0013, issue 14 in the plan: hooks are functions the evaluator from issue
  14 returns (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 17 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #61, "Evaluate a configuration, with the surface v0.1.0 accepts",
carried the work, matched on "issues 14 through 18" in its body:
https://github.com/meowshed/meowctl/pull/61 (from
https://github.com/meowshed/meowctl/pull/61, high).

## Left alone

Nothing recorded.
