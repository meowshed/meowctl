---
id: TSK-0029
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3040, REQ-3041, REQ-3042, REQ-3043, REQ-3044, REQ-3116]
issue:
---

# Sentinel and staleness

This task records issue 31 of the 0.2.0 execution plan, which the plan marks
done, and closes 6 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a module whose version changed since the last run, when the next run
   plans, then its components are no longer skipped as complete (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-config/tests/parity.rs`,
   `crates/meowctl-engine/tests/running.rs`,
   `crates/meowctl-engine/tests/sentinel.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Record completed components in the sentinel and clear them when a module changes
(from https://github.com/meowshed/meowctl/pull/75, medium). Staleness is where
`v0.1.0` was already fixed once, so the issue includes a test reproducing the
bug `fix: correct module updates` closed (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0004, issue 5 in the plan: completions are written to `state.toml`
  (inferred from docs/design/0.2.0-execution-plan.md, medium).
- TSK-0005, issue 6 in the plan: staleness compares fingerprints in
  `installed.lock` (inferred from docs/design/0.2.0-execution-plan.md, medium).
- TSK-0028, issue 30 in the plan: completions are recorded as phases run
  (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 31 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #75, "feat(engine): record what finished, and redo what a module
changed", carried the work, matched on "Issue 31" in its body:
https://github.com/meowshed/meowctl/pull/75 (from
https://github.com/meowshed/meowctl/pull/75, high).

## Left alone

Nothing recorded.
