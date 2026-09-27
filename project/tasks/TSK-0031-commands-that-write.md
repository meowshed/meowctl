---
id: TSK-0031
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3403, REQ-3500]
issue: 129
projected: cfdcd758bddd
---

# The commands that write a configuration

This task records issue 35 of the 0.2.0 execution plan, which the plan marks
done, and closes 2 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given an empty directory, when a user runs `meowctl init`, then it scaffolds
   a configuration directory (inferred from docs/design/0.2.0-execution-plan.md,
   medium). Closed by: the tests that cite its requirements, in
   `crates/meowctl-cli/tests/surface.rs` (from `rg REQ- crates tests`, medium).

## What to do

Add `init`, `add`, `remove`, `update`, `self-update`, `dep add`, `dep remove`
and `dep upgrade` (from docs/design/0.2.0-execution-plan.md, high). Each edits a
configuration file and then runs an apply, so they share one shape that differs
from everything in issue 32; bundled with it they would make one issue nobody
could review (from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0030, issue 32 in the plan: these commands run an apply through the tree
  issue 32 builds (from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 35 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #77, "feat(cli): scaffold, edit and resolve a configuration",
carried the work, matched on "Issue 35" in its body:
https://github.com/meowshed/meowctl/pull/77 (from
https://github.com/meowshed/meowctl/pull/77, high).

## Left alone

Nothing recorded.
