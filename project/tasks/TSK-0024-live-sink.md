---
id: TSK-0024
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3204, REQ-3220, REQ-3221, REQ-3222, REQ-3223, REQ-3224, REQ-3225, REQ-3226, REQ-3281, REQ-3305, REQ-3306, REQ-3307]
issue:
---

# The live sink

This task records issue 26 of the 0.2.0 execution plan, which the plan marks
done, and closes 12 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a terminal and a running command, when a subprocess asks for the
   terminal, then the live sink erases its region, yields the terminal and
   redraws after (inferred from docs/design/0.2.0-execution-plan.md, medium).
   Closed by: the tests that cite its requirements, in
   `crates/meowctl-tui/tests/live.rs` (from `rg REQ- crates tests`, medium).

## What to do

Add the live sink: region erasure, cursor arithmetic, terminal hand-off and the
capability matrix in snapshots (from docs/design/0.2.0-execution-plan.md, high).
It is the subtlest code in the tree and the least splittable, because half a
live renderer draws a region it cannot erase (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0023, issue 25 in the plan: the live sink follows the two that need no
  terminal (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 26 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #71, "feat(tui): redraw a region in place, and give the terminal
back", carried the work, matched on "Issue 26" in its body:
https://github.com/meowshed/meowctl/pull/71 (from
https://github.com/meowshed/meowctl/pull/71, high).

## Left alone

Nothing recorded.
