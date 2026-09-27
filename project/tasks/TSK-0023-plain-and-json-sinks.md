---
id: TSK-0023
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3201, REQ-3202, REQ-3203, REQ-3210, REQ-3211, REQ-3212, REQ-3230, REQ-3231, REQ-3232, REQ-3270, REQ-3271, REQ-3280, REQ-3300, REQ-3301, REQ-3302, REQ-3303, REQ-3323]
issue: 121
projected: 71777ce01f15
---

# The plain and JSON sinks

This task records issue 25 of the 0.2.0 execution plan, which the plan marks
done, and closes 17 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a recorded event stream, when the JSON sink renders it, then it writes
   one object per line with no decoration (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-cli/tests/surface.rs`,
   `crates/meowctl-common/src/event.rs`, `crates/meowctl-tui/src/theme.rs`,
   `crates/meowctl-tui/tests/rendering.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Add the plain and JSON sinks (from docs/design/0.2.0-execution-plan.md, medium).
The two sinks that need no terminal come first, because they are what the corpus
and the tests use (from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0022, issue 24 in the plan: a sink draws with the capabilities and palette
  from issue 24 (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 25 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #64, "Render the event stream, with no terminal and no engine",
carried the work, matched on "Issues 24 and 25" in its body:
https://github.com/meowshed/meowctl/pull/64 (from
https://github.com/meowshed/meowctl/pull/64, high).

## Left alone

The capability matrix in snapshots, which waited for `insta` and landed with the
live sink (from https://github.com/meowshed/meowctl/pull/64, high).
