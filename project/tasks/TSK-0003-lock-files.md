---
id: TSK-0003
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1220, REQ-1221, REQ-1222, REQ-1223, REQ-1224, REQ-1225, REQ-1261, REQ-1305, REQ-1306, REQ-1307, REQ-1308, REQ-1309]
issue: 101
projected: 8a241505604d
---

# Lock files

This task records issue 4 of the 0.2.0 execution plan, which the plan marks
done, and closes 12 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a lock file `v0.1.0` wrote, when `meowctl-config` reads it and writes
   it back, then the written bytes match the fixture outside the `meta` table
   (from docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that
   cite its requirements, in `crates/meowctl-cli/src/run/commands.rs`,
   `crates/meowctl-config/tests/parity.rs`,
   `crates/meowctl-module/tests/syncing.rs`, `tests/pkgs_lock.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Read and write `deps.lock`, `deps.local.lock`, `pkgs.lock` and
`pkgs.local.lock`, checked against the M1 fixtures byte for byte (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0001, issue 2 in the plan: the lock records module references and
  integrity hashes, which issue 2 defines (inferred from
  docs/design/0.2.0-execution-plan.md, medium).
- TSK-0009, issue 10 in the plan: every configuration write goes through
  `FileSystem` (from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 4 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #60, "Write the on-disk formats exactly as v0.1.0 writes them",
carried the work, matched on "Issues 4, 5, 6 and 9" in its body:
https://github.com/meowshed/meowctl/pull/60 (from
https://github.com/meowshed/meowctl/pull/60, high).

## Left alone

Writing `pkgs.lock`. The file shares the lock schema, so its format is covered,
but nothing wrote one until the package-manager dispatch existed (from
https://github.com/meowshed/meowctl/pull/60, high); issue 38 later made a run
write it (from docs/design/0.2.0-execution-plan.md, high).
