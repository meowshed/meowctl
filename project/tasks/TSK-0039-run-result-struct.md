---
id: TSK-0039
artifact: task
status: done
revised: 2026-10-25
bug: BUG-0041
closes: []
---

# Return a struct from `ctx.run`

`ctx.run` returns a struct carrying `stdout`, `stderr` and `exit_code` as
attributes, as `v0.1.0` did, so the corpus reads the fields again.

## Acceptance criteria

1. Given a scripted run that exits 1 with `not installed` on stderr, when a
   hook calls `ctx.run` and reads `result.exit_code`, `result.stderr` and
   `result.stdout`, then it reads 1, `not installed` and the empty string.
   Closed by: `run_returns_three_fields_and_does_not_fail_on_a_non_zero_exit`
   in `crates/meowctl-ctx/tests/surface.rs`, rewritten to read attributes.
2. Given the engine tests that parse `ctx.run` output, when they read
   `result.stdout`, then they pass. Closed by: the two helpers in
   `crates/meowctl-engine/tests/running.rs`, rewritten to read attributes.

## What to do

Change `run_result` in `crates/meowctl-ctx/src/value.rs` to allocate a
starlark-rust struct in place of a dict, and rewrite the tests that read the
result by index to read attributes, because index access never worked in
`v0.1.0` and no corpus component uses it. Keep the field names and types
[REQ-2820] fixes.

## Depends on

None.

## Left alone

Index access, `result["stdout"]`, stops working, because `v0.1.0` never
offered it and no corpus component uses it.

## Evidence

Criterion 1 is closed by
`run_returns_three_fields_and_does_not_fail_on_a_non_zero_exit`, and criterion
2 by `query_pm_returns_what_the_handler_reports` and
`interrogate_runs_its_command_in_a_read_only_phase_of_a_dry_run`. All three
failed on the dict with "Object of type `dict` has no attribute" before the
fix, and pass after it.

At `fix/run-result-struct`, `mise run fmt-check`, `mise run check`,
`mise run test` and `mise run build` each exited 0, and the test stage ran 600
tests with 600 passing and 3 skipped. The corpus test
`crates/meowctl-starlark/tests/stdlib.rs` ran against the meowctl-stdlib and
dotmeow checkouts beside the repository.
