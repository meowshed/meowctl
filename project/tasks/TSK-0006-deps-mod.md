---
id: TSK-0006
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1210, REQ-1211, REQ-1212, REQ-1213, REQ-1214, REQ-1302, REQ-1303, REQ-1304]
issue: 104
projected: 64b09dc37332
---

# deps.mod reading and writing

This task records issue 7 of the 0.2.0 execution plan, which the plan marks
done, and closes 8 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a set of `dep()`, `module()` and `replace()` declarations, when
   `meowctl-config` writes them as `deps.mod`, then the file matches the layout
   `v0.1.0` writes byte for byte (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-config/src/modfile.rs`,
   `crates/meowctl-config/tests/editing.rs`,
   `crates/meowctl-config/tests/parity.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Write `deps.mod` in the byte-exact layout `v0.1.0` uses (from
docs/design/0.2.0-execution-plan.md, high). Reading it is evaluating it:
`dep()`, `module()` and `replace()` are in the accumulator, and the engine hands
what comes back to `meowctl-config` (from docs/design/0.2.0-execution-plan.md,
high).

`meowctl-config` gains no dependency on `meowctl-starlark` for this (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0001, issue 2 in the plan: the manifest records module references from
  issue 2 (inferred from docs/design/0.2.0-execution-plan.md, medium).
- TSK-0015, issue 16 in the plan: reading `deps.mod` is evaluating it, which
  needs the evaluator (from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 7 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #63, "Edit a Starlark file by parsing it, not by matching text",
carried the work, matched on "Issues 7 and 8" in its body:
https://github.com/meowshed/meowctl/pull/63 (from
https://github.com/meowshed/meowctl/pull/63, high).

## Left alone

`meowctl-config` does not read `deps.mod`. Editing needs the syntax tree and
evaluation needs the builtins and the accumulator, so the two directions live in
different crates on purpose (from https://github.com/meowshed/meowctl/pull/63,
high).
