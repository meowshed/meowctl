---
id: TSK-0002
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1040, REQ-1041, REQ-1042, REQ-1043, REQ-1109, REQ-1110]
issue:
---

# The event vocabulary

This task records issue 3 of the 0.2.0 execution plan, which the plan marks
done, and closes 6 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given one value of each `Event` variant, when a test serialises each to JSON,
   then each serialises to its JSON form (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-common/src/event.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Add the `Event` enum and its JSON form (from
docs/design/0.2.0-execution-plan.md, high). `Event` lives in `meowctl-common` so
that a sink never depends on the engine that produces the events (from
https://github.com/meowshed/meowctl/pull/56, high).

## Depends on

- TSK-0001, issue 2 in the plan: the events carry the identifiers and phases
  issue 2 defines (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 3 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #56, "Add the shared vocabulary, and stop the corpus running
foreign hooks", carried the work, matched on "Issues 2 and 3" in its body:
https://github.com/meowshed/meowctl/pull/56 (from
https://github.com/meowshed/meowctl/pull/56, high).

## Left alone

Nothing recorded.
