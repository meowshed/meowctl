---
id: REQ-1770
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1770

A command `DryRunExecutor` does not run MUST return exit code 0 with empty
standard output and standard error.

A hook that branches on failure would otherwise take the failure path for
every command in a dry run and plan a repair that isn't needed (from
https://github.com/meowshed/meowctl/pull/58 and
crates/meowctl-exec/src/dry_run.rs:58-65, high). It elaborates REQ-1700, which
says the command doesn't run and says nothing about what the caller gets back.

Added during onboarding from the research on decision 6; the shipped
`DryRunExecutor` returns `exit_code: Some(0)` with empty output (from
crates/meowctl-exec/src/dry_run.rs:58-65, high).
