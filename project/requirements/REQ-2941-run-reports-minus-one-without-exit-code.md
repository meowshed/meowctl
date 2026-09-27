---
id: REQ-2941
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2941

`ctx.run` MUST report `exit_code` as `-1` for a process that ended without an
exit code.

Components compare against `-1`, and `v0.1.0` returns it, because Go's
`ExitCode()` does for a process killed by a signal (from
crates/meowctl-ctx/src/value.rs:262-280, medium: the parity claim is a test
comment).

Added during onboarding from the failure-path research; the shipped method
returns `-1`, and no test covers it (from
crates/meowctl-ctx/src/value.rs:262-280, high).
