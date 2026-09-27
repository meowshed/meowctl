---
id: TSK-0013
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2201, REQ-2202, REQ-2203, REQ-2204, REQ-2206, REQ-2207, REQ-2208, REQ-2300, REQ-2301, REQ-2302, REQ-2303]
issue: 111
projected: b7d780d7ea0d
---

# The evaluator and the predeclared set

This task records issue 14 of the 0.2.0 execution plan, which the plan marks
done, and closes 11 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given every component file in `meowctl-stdlib`, when the evaluator evaluates
   it, then it evaluates to the declarations `v0.1.0` produces (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-starlark/src/platform.rs`,
   `crates/meowctl-starlark/tests/evaluation.rs`,
   `crates/meowctl-starlark/tests/stdlib.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Add the Starlark evaluator with the eleven predeclared builtins and the `json`
module (from docs/design/0.2.0-execution-plan.md, high). The issue is large on
purpose: the builtins share the accumulator and the argument-parsing shape, and
split apart they would be eleven issues that each change one match arm (from
docs/design/0.2.0-execution-plan.md, high).

A later change let `module()` accept `compat`, because every `MODULE.meow` in
the meowshed organisation carries it and the evaluator rejected it (from
https://github.com/meowshed/meowctl/pull/66, high).

## Depends on

- TSK-0001, issue 2 in the plan: the builtins record identifiers and phases from
  issue 2 (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 14 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #61, "Evaluate a configuration, with the surface v0.1.0 accepts",
carried the work, matched on "issues 14 through 18" in its body:
https://github.com/meowshed/meowctl/pull/61 (from
https://github.com/meowshed/meowctl/pull/61, high).

Pull request #66, "Read a module's own manifest, not just meowctl's", carried
the work, amended the `module()` requirement this issue closes:
https://github.com/meowshed/meowctl/pull/66 (from
https://github.com/meowshed/meowctl/pull/66, high).

## Left alone

`query_pm`. The plan records it under this issue and it was left out, because it
runs user code during evaluation and needs a registry to ask; issue 22 closes it
(from docs/design/0.2.0-execution-plan.md, high).

The extended global set. The evaluator uses starlark-rust's standard set,
because the extended one adds builtins `v0.1.0` does not provide (from
https://github.com/meowshed/meowctl/pull/61, high).
