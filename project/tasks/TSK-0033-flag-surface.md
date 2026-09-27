---
id: TSK-0033
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3415, REQ-3416, REQ-3508, REQ-3509]
issue:
---

# The flag surface v0.1.0 actually has

This task records issue 37 of the 0.2.0 execution plan, which the plan marks
done, and closes 4 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given the `v0.1.0` flag table, when the tests walk every command, then each
   command accepts exactly the flags `v0.1.0` gives it (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-cli/tests/surface.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Give each command the flags `v0.1.0` gives it. `addLifecycleFlags` gives
`--dry-run`, `--no-rollback` and `--verbose` to all six lifecycle commands, and
`addInstallFlags` gives `--force` and `--ignore-lock` to `apply`, `add` and
`upgrade` (from docs/design/0.2.0-execution-plan.md, high). The test walks the
table, because the old test checked three flags on `apply` alone (from
docs/design/0.2.0-execution-plan.md, high).

`--ignore-lock` reaches no code path in either binary. It stays, because
dropping it would break a script that passes it, and the requirement says it is
inert (from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0032, issue 36 in the plan: the plan blocks it on issue 36 (from
  docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 37 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #83, "fix(cli): give each command the flags v0.1.0 gives it",
carried the work, matched on the flag requirements it cites:
https://github.com/meowshed/meowctl/pull/83 (from
https://github.com/meowshed/meowctl/pull/83, high).

## Left alone

Nothing recorded.
