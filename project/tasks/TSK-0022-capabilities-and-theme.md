---
id: TSK-0022
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3240, REQ-3241, REQ-3242, REQ-3243, REQ-3244, REQ-3250, REQ-3251, REQ-3252, REQ-3308, REQ-3309, REQ-3310, REQ-3313]
issue:
---

# Capability detection and the theme

This task records issue 24 of the 0.2.0 execution plan, which the plan marks
done, and closes 12 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given `NO_COLOR` set, or output to a pipe, when capabilities are detected,
   then colour or motion is turned off as the requirements say (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-tui/src/caps.rs`,
   `crates/meowctl-tui/src/theme.rs`, `crates/meowctl-tui/tests/rendering.rs`,
   `tests/theme.rs` (from `rg REQ- crates tests`, medium).

## What to do

Detect the terminal's capabilities and make the palette data (from
docs/design/0.2.0-execution-plan.md, medium).

## Depends on

- TSK-0002, issue 3 in the plan: the theme styles the events issue 3 defines
  (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 24 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #64, "Render the event stream, with no terminal and no engine",
carried the work, matched on "Issues 24 and 25" in its body:
https://github.com/meowshed/meowctl/pull/64 (from
https://github.com/meowshed/meowctl/pull/64, high).

## Left alone

The theme file. The palette became data here, and loading it from disk waited;
issue 45 closed it (from https://github.com/meowshed/meowctl/pull/64, high).
