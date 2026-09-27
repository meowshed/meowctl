---
id: TSK-0035
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3064, REQ-3453, REQ-3454, REQ-3519]
issue: 133
projected: 5e872c2fcf43
---

# Stopping a run

This task records issue 40 of the 0.2.0 execution plan, which the plan marks
done, and closes 4 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a running `apply`, when a signal is sent to it, then it stops before
   the next component, leaves the journal and exits 1 (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-engine/tests/running.rs`,
   `tests/interrupt.rs` (from `rg REQ- crates tests`, medium).

## What to do

Let the first interrupt set a flag the CLI owns, which the runner reads between
components, and let the second kill the process (from
docs/design/0.2.0-execution-plan.md, high). Before this, the handler killed the
process where it stood and the engine never saw the interrupt (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0032, issue 36 in the plan: the plan blocks it on issue 36 (from
  docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 40 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #86, "feat(cli): let an interrupt stop the run instead of killing
it", carried the work, matched on the interrupt requirements it cites:
https://github.com/meowshed/meowctl/pull/86 (from
https://github.com/meowshed/meowctl/pull/86, high).

## Left alone

Stopping a component part-way. A hook halfway through `brew install` is not the
engine's to stop, and a terminal's interrupt reaches the process group anyway
(from docs/design/0.2.0-execution-plan.md, high).
