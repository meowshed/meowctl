---
id: TSK-0008
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1201, REQ-1202, REQ-1203, REQ-1204, REQ-1260, REQ-1262, REQ-1263, REQ-1300, REQ-1301, REQ-1315]
issue:
---

# Config layout and the legacy check

This task records issue 9 of the 0.2.0 execution plan, which the plan marks
done, and closes 10 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a configuration file being written, when the write fails part-way, then
   the previous file is left in place (from docs/design/0.2.0-execution-plan.md,
   medium). Closed by: the tests that cite its requirements, in
   `crates/meowctl-config/tests/atomicity.rs`,
   `crates/meowctl-config/tests/parity.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Resolve the configuration layout and report a legacy layout (from
docs/design/0.2.0-execution-plan.md, medium).

Three requirements added here were claimed by no issue and are properties of
every format: writes go through `FileSystem`, a failed write leaves the previous
file, and two runs cannot interleave. The atomic rename in `meowctl-fs` provides
all three, so the work is the tests that say so (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0001, issue 2 in the plan: the layout resolves paths with the path rules
  issue 2 defines (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 9 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #60, "Write the on-disk formats exactly as v0.1.0 writes them",
carried the work, matched on "Issues 4, 5, 6 and 9" in its body:
https://github.com/meowshed/meowctl/pull/60 (from
https://github.com/meowshed/meowctl/pull/60, high).

Pull request #67, "Fetch a module, verify it, and know when the cache is lying",
carried the work, claimed the three unclaimed requirements this issue closes:
https://github.com/meowshed/meowctl/pull/67 (from
https://github.com/meowshed/meowctl/pull/67, high).

## Left alone

Nothing recorded.
