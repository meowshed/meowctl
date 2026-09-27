---
id: TSK-0018
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2401, REQ-2402, REQ-2403, REQ-2404, REQ-2405, REQ-2461, REQ-2462, REQ-2500]
issue:
---

# Version selection

This task records issue 20 of the 0.2.0 execution plan, which the plan marks
done, and closes 8 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a requirement graph, when selection runs twice, then each module gets
   the maximum version any requirement names, and both runs agree (inferred from
   docs/design/0.2.0-execution-plan.md, medium). Closed by: the tests that cite
   its requirements, in `crates/meowctl-module/src/mvs.rs`,
   `crates/meowctl-module/src/version.rs`,
   `crates/meowctl-module/tests/fetching.rs`,
   `crates/meowctl-module/tests/syncing.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Select module versions with Minimal Version Selection, as `v0.1.0` does (from
https://github.com/meowshed/meowctl/pull/62, high). The Go tests in
`internal/mvs/mvs_test.go` port directly and should (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0001, issue 2 in the plan: versions are selected over the module
  references issue 2 defines (inferred from docs/design/0.2.0-execution-plan.md,
  medium).

## Evidence

The plan marks issue 20 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #62, "Select versions the way v0.1.0 does, reproducibly", carried
the work, matched on "Issue 20" in its body:
https://github.com/meowshed/meowctl/pull/62 (from
https://github.com/meowshed/meowctl/pull/62, high).

## Left alone

A line-by-line port of the Go tests. They test the same algorithm through a
different interface, and the Rust tests cover the same cases (from
https://github.com/meowshed/meowctl/pull/62, high).
