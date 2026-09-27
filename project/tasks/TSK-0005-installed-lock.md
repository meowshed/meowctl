---
id: TSK-0005
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1230, REQ-1231, REQ-1232, REQ-1233]
issue:
---

# installed.lock, both schema versions

This task records issue 6 of the 0.2.0 execution plan, which the plan marks
done, and closes 4 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given an `installed.lock` fixture in the version 1 form, when
   `meowctl-config` parses it, then it parses, and a write produces the version
   2 form (from docs/design/0.2.0-execution-plan.md, medium). Closed by: the
   tests that cite its requirements, in `crates/meowctl-config/tests/parity.rs`
   (from `rg REQ- crates tests`, medium).

## What to do

Read and write `installed.lock` in both schema versions. The version 1 form must
still parse, and a fixture in the old form is part of the issue (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0001, issue 2 in the plan: the lock records component identifiers from
  issue 2 (inferred from docs/design/0.2.0-execution-plan.md, medium).
- TSK-0009, issue 10 in the plan: every configuration write goes through
  `FileSystem` (from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 6 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #60, "Write the on-disk formats exactly as v0.1.0 writes them",
carried the work, matched on "Issues 4, 5, 6 and 9" in its body:
https://github.com/meowshed/meowctl/pull/60 (from
https://github.com/meowshed/meowctl/pull/60, high).

## Left alone

Nothing recorded.
