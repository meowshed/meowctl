---
id: TSK-0034
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1226, REQ-1310, REQ-1311]
issue:
---

# The package locks

This task records issue 38 of the 0.2.0 execution plan, which the plan marks
done, and closes 4 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a run that declares packages, and one that declares none, when each run
   finishes, then the first writes both files split by declaring entry point,
   and the second writes neither (from docs/design/0.2.0-execution-plan.md,
   high). Closed by: the tests that cite its requirements, in
   `tests/pkgs_lock.rs` (from `rg REQ- crates tests`, medium).

## What to do

Write `pkgs.lock` and `pkgs.local.lock` at the end of a run, as `v0.1.0` ends
`runApply` with `appendPkgsLock` (from docs/design/0.2.0-execution-plan.md,
high).

The engine collects and the CLI writes: `Report` gains the declarations of every
component whose `install` or `upgrade` ran, and nothing below `meowctl-cli`
learns where the configuration directory is (from
docs/design/0.2.0-execution-plan.md, high). The declaring entry point decides
which file a component lands in, and getting it backwards commits a machine's
private packages to a shared repository (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0032, issue 36 in the plan: the plan blocks it on issue 36 (from
  docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 38 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #84, "feat(cli): record what a run installed in the package locks",
carried the work, matched on the package-lock requirements it cites:
https://github.com/meowshed/meowctl/pull/84 (from
https://github.com/meowshed/meowctl/pull/84, high).

## Left alone

Nothing recorded.
