---
id: REQ-3140
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3140

A `local.star` that exists and can't be read or evaluated MUST stop the command
before any phase runs.

A machine overlay that is skipped or half evaluated would apply the shared
configuration without the local one, so a file that can't be read is a failure,
not an absence.

Added during onboarding from the failure-path research; the shipped binary stops
on an evaluation error with the configuration code, and skips a file it can't
read as if it were absent (from crates/meowctl-cli/src/run/commands.rs:259-289
and commands.rs:264-266, high).
