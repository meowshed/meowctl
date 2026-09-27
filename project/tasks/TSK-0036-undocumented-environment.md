---
id: TSK-0036
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1606, REQ-3245, REQ-3246, REQ-3311, REQ-3312]
issue:
---

# The environment nothing had written down

This task records issue 42 of the 0.2.0 execution plan, which the plan marks
done, and closes 5 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given each of the three variables, when the tests run, then each has a
   requirement and a test that holds it (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-exec/src/lib.rs`,
   `crates/meowctl-tui/src/caps.rs` (from `rg REQ- crates tests`, medium).

## What to do

Specify the three environment variables that change observable behaviour:
`CLICOLOR_FORCE` turns colour back on for a pipe, `COLORTERM` selects the 24-bit
depth, and `PATHEXT` decides whether `git` resolves to `git.exe` (from
docs/design/0.2.0-execution-plan.md, high).

Compute the `PATHEXT` order in a pure function of the name and the string, so it
can be tested on every platform without mutating the process environment, which
edition 2024 makes `unsafe` (from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0032, issue 36 in the plan: the plan blocks it on issue 36 (from
  docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 42 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #88, "docs(spec): write down the environment that was changing
behaviour", carried the work, matched on the environment requirements it cites:
https://github.com/meowshed/meowctl/pull/88 (from
https://github.com/meowshed/meowctl/pull/88, high).

## Left alone

Nothing recorded.
