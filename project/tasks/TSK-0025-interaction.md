---
id: TSK-0025
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3260, REQ-3261, REQ-3262, REQ-3319, REQ-3320]
issue: 123
projected: 96fcc6ad156b
---

# Interaction

This task records issue 27 of the 0.2.0 execution plan, which the plan marks
done, and closes 5 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a confirmation with no terminal attached, when it is asked, then it
   fails, and end of input answers no (from
   https://github.com/meowshed/meowctl/pull/65, medium). Closed by: the tests
   that cite its requirements, in `crates/meowctl-tui/src/interaction.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Add the `Interaction` trait's confirmation, so prompting leaves the printer
(from https://github.com/meowshed/meowctl/pull/65, high).

## Depends on

- TSK-0022, issue 24 in the plan: a prompt writes with the capabilities issue 24
  detects (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 27 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #65, "Ask the user without blocking a machine", carried the work,
matched on "Issue 27" in its body: https://github.com/meowshed/meowctl/pull/65
(from https://github.com/meowshed/meowctl/pull/65, high).

## Left alone

Wiring `Always` to `--yes`, which is the CLI's (from
https://github.com/meowshed/meowctl/pull/65, high).
