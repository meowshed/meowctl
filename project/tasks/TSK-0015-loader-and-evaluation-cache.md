---
id: TSK-0015
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2223, REQ-2253]
issue:
---

# The Loader trait and evaluation caching

This task records issue 16 of the 0.2.0 execution plan, which the plan marks
done, and closes 2 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a module URL loaded twice in one run, when both `load()` calls resolve,
   then the module is evaluated once (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-starlark/tests/evaluation.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Add the `Loader` trait, the evaluation cache and the load chain in diagnostics
(from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0013, issue 14 in the plan: `load()` is resolved inside the evaluator
  issue 14 adds (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 16 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #61, "Evaluate a configuration, with the surface v0.1.0 accepts",
carried the work, matched on "issues 14 through 18" in its body:
https://github.com/meowshed/meowctl/pull/61 (from
https://github.com/meowshed/meowctl/pull/61, high).

## Left alone

The composite loader. The plan first recorded this issue as closing the two
`load()` resolution requirements, and it did not: resolving the four schemes
needs the fetching issue 19 builds, so they land there (from
docs/design/0.2.0-execution-plan.md, high).

Integrity before evaluation, which lands with issue 19 because that is what can
verify it (from docs/design/0.2.0-execution-plan.md, high).
