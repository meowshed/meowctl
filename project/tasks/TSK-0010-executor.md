---
id: TSK-0010
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1601, REQ-1602, REQ-1603, REQ-1604, REQ-1605, REQ-1610, REQ-1611, REQ-1612, REQ-1620, REQ-1621, REQ-1622, REQ-1623, REQ-1630, REQ-1631, REQ-1632, REQ-1700, REQ-1701, REQ-1702, REQ-1703, REQ-1704]
issue:
---

# Executor and terminal hand-off

This task records issue 11 of the 0.2.0 execution plan, which the plan marks
done, and closes 20 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given an interactive command, when the executor runs it, then it emits the
   events that hand the terminal over and take it back, and holds no renderer
   (inferred from docs/design/0.2.0-execution-plan.md, medium). Closed by: the
   tests that cite its requirements, in
   `crates/meowctl-engine/tests/running.rs`,
   `crates/meowctl-exec/tests/execution.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Put every subprocess behind the `Executor` trait, with the events that hand the
terminal to an interactive command (from docs/design/0.2.0-execution-plan.md,
medium). The events are here; the sink that consumes them is issue 24 (from
docs/design/0.2.0-execution-plan.md, high).

`SuspendOutput` is deleted, so no builtin can reach back and stand the renderer
down (from https://github.com/meowshed/meowctl/pull/58, high).

## Depends on

- TSK-0001, issue 2 in the plan: commands carry the error taxonomy issue 2
  defines (inferred from docs/design/0.2.0-execution-plan.md, medium).
- TSK-0002, issue 3 in the plan: terminal hand-off is expressed as events from
  the vocabulary issue 3 defines (inferred from
  docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 11 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #58, "Put every subprocess behind one trait, and delete
SuspendOutput", carried the work, matched on "Issue 11" in its body:
https://github.com/meowshed/meowctl/pull/58 (from
https://github.com/meowshed/meowctl/pull/58, high).

## Left alone

`ctx.git_clone`, which builds a `git` command and runs it through this trait, so
it is not an executor concern (from https://github.com/meowshed/meowctl/pull/58,
high).
