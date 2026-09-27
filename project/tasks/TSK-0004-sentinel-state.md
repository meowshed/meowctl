---
id: TSK-0004
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1240, REQ-1241, REQ-1242, REQ-1243, REQ-1244, REQ-1312, REQ-1313]
issue:
---

# Sentinel state

This task records issue 5 of the 0.2.0 execution plan, which the plan marks
done, and closes 7 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a `state.toml` carrying a schema version newer than this binary's, when
   a run tries to write the state, then the write is refused and the file is
   unchanged (from docs/design/0.2.0-execution-plan.md, high). Closed by: the
   tests that cite its requirements, in `crates/meowctl-common/src/phase.rs`,
   `crates/meowctl-config/tests/parity.rs`,
   `crates/meowctl-ops/tests/journal.rs` (from `rg REQ- crates tests`, medium).

## What to do

Read and write `state.toml`, the sentinel state (from
docs/design/0.2.0-execution-plan.md, high).

Refuse to overwrite a `state.toml` with a newer schema version. The decisions
document settles this in §3.8, and the issue needs a test that writes a future
version and asserts the refusal (from docs/design/0.2.0-execution-plan.md,
high).

## Depends on

- TSK-0001, issue 2 in the plan: the state records phases and component
  identifiers from issue 2 (inferred from docs/design/0.2.0-execution-plan.md,
  medium).
- TSK-0009, issue 10 in the plan: every configuration write goes through
  `FileSystem` (from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 5 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #60, "Write the on-disk formats exactly as v0.1.0 writes them",
carried the work, matched on "Issues 4, 5, 6 and 9" in its body:
https://github.com/meowshed/meowctl/pull/60 (from
https://github.com/meowshed/meowctl/pull/60, high).

## Left alone

Nothing recorded.
