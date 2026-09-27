---
id: TSK-0009
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1401, REQ-1402, REQ-1403, REQ-1404, REQ-1410, REQ-1411, REQ-1412, REQ-1413, REQ-1414, REQ-1420, REQ-1421, REQ-1422, REQ-1430, REQ-1431, REQ-1432, REQ-1433, REQ-1500, REQ-1504, REQ-1505, REQ-1506, REQ-1507, REQ-1508]
issue: 107
projected: 980acd063555
---

# FileSystem and its three implementations

This task records issue 10 of the 0.2.0 execution plan, which the plan marks
done, and closes 22 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a dry-run filesystem, when a caller writes a file and reads it back,
   then the read returns the written bytes and nothing on disk changes (inferred
   from docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that
   cite its requirements, in `crates/meowctl-config/tests/atomicity.rs`,
   `crates/meowctl-ctx/tests/surface.rs`,
   `crates/meowctl-fs/tests/conformance.rs`,
   `crates/meowctl-fs/tests/dry_run.rs`, `tests/dry_run.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Put every filesystem effect behind the `FileSystem` trait, with a real, a
dry-run and an in-memory implementation (from
docs/design/0.2.0-execution-plan.md, medium).

Two decided requirements are part of it: a dry run reads back its own writes,
and a dry run predicts failures a real run would hit. They make a dry run a
prediction of what would happen, where a description would only list the steps
(from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0001, issue 2 in the plan: the trait takes the resolved paths issue 2
  defines (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 10 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #57, "Put every filesystem effect behind one trait", carried the
work, matched on "Issue 10" in its body:
https://github.com/meowshed/meowctl/pull/57 (from
https://github.com/meowshed/meowctl/pull/57, high).

## Left alone

`Op` and the journal, which give these effects their inverses; they are issues
12 and 13 (from https://github.com/meowshed/meowctl/pull/57, high).
