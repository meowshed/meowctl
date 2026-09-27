---
id: TSK-0001
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1001, REQ-1002, REQ-1003, REQ-1004, REQ-1010, REQ-1011, REQ-1012, REQ-1013, REQ-1020, REQ-1021, REQ-1022, REQ-1030, REQ-1031, REQ-1032, REQ-1050, REQ-1051, REQ-1100, REQ-1101, REQ-1102, REQ-1103, REQ-1104, REQ-1105, REQ-1107, REQ-1108]
issue:
---

# Domain types and the error taxonomy

This task records issue 2 of the 0.2.0 execution plan, which the plan marks
done, and closes 24 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given the Rust workspace, when a user runs `meowctl version` and
   `meowctl --help`, then each prints its output and exits with the code
   `v0.1.0` uses (from docs/design/0.2.0-execution-plan.md, high). Closed by:
   the tests that cite its requirements, in
   `crates/meowctl-common/src/error.rs`, `crates/meowctl-common/src/id.rs`,
   `crates/meowctl-common/src/paths.rs`, `crates/meowctl-common/src/phase.rs`,
   `crates/meowctl-ctx/tests/surface.rs`, `tests/errors.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Add the identifiers, phases, paths and exit codes that every other crate uses,
in `meowctl-common` (from docs/design/0.2.0-execution-plan.md, high). The plan
marks it as blocking everything, because each later issue names these types
(from docs/design/0.2.0-execution-plan.md, high).

The slice that makes it observable is `meowctl version` and `meowctl --help`
running from the Rust binary and exiting with the right code (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

Nothing: the plan gives issue 2 no blockers (from
docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 2 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #56, "Add the shared vocabulary, and stop the corpus running
foreign hooks", carried the work, matched on "Issues 2 and 3" in its body:
https://github.com/meowshed/meowctl/pull/56 (from
https://github.com/meowshed/meowctl/pull/56, high).

## Left alone

The `meowctl` binary. A binary that answered only `version` would have made the
corpus check fail for every other command while proving what the exit-code test
already proves, so the observable slice waited for the CLI issue (from
https://github.com/meowshed/meowctl/pull/56, high).
