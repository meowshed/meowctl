---
id: TSK-0014
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2210, REQ-2211, REQ-2212]
issue: 112
projected: d46484d35e88
---

# The accumulator

This task records issue 15 of the 0.2.0 execution plan, which the plan marks
done, and closes 3 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a file declaring several components, when it is evaluated, then the
   accumulator returns owned declarations in declaration order (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-starlark/tests/evaluation.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Collect declarations per evaluation in an accumulator that holds owned data and
keeps declaration order (from docs/design/0.2.0-execution-plan.md, medium). M0
made owned data a compiler-enforced fact, so the work is the owned declarations
and the ordering (from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0013, issue 14 in the plan: the builtins issue 14 adds are what write into
  the accumulator (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 15 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #61, "Evaluate a configuration, with the surface v0.1.0 accepts",
carried the work, matched on "issues 14 through 18" in its body:
https://github.com/meowshed/meowctl/pull/61 (from
https://github.com/meowshed/meowctl/pull/61, high).

## Left alone

Nothing recorded.
