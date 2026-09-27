---
id: TSK-0012
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2020, REQ-2021, REQ-2022, REQ-2023, REQ-2024, REQ-2025, REQ-2026, REQ-2030, REQ-2031, REQ-2032, REQ-2116, REQ-2117, REQ-2118, REQ-2119, REQ-2120]
issue: 110
projected: de02f8d13489
---

# The write-ahead journal

This task records issue 13 of the 0.2.0 execution plan, which the plan marks
done, and closes 15 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a journal the `v0.1.0` binary wrote, when the Rust implementation
   replays it, then each recorded inverse runs in reverse order (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-ctx/tests/surface.rs`,
   `crates/meowctl-ops/tests/inverses.rs`,
   `crates/meowctl-ops/tests/journal.rs`, `tests/dry_run.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Add the write-ahead journal of operations (from
docs/design/0.2.0-execution-plan.md, medium). The format must interoperate with
`v0.1.0`'s, so the issue includes replaying a journal the Go binary wrote (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0011, issue 12 in the plan: the journal is a log of the operations issue
  12 defines (inferred from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 13 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #59, "Make every effect a value with an inverse, and journal it",
carried the work, matched on "Issues 12 and 13" in its body:
https://github.com/meowshed/meowctl/pull/59 (from
https://github.com/meowshed/meowctl/pull/59, high).

## Left alone

Nothing recorded.
