---
id: REQ-3518
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3518

An unknown command MUST suggest the nearest command when one is close.

A mistyped command is the most common usage error, and naming the nearest one
lets the user correct it without reading `--help` (inferred during onboarding,
since no source states a reason, low). The source also named an unknown flag,
but clap suggests a flag and not a command for one, and the test holds only the
command half, so [REQ-3590] carries the flag half (from
crates/meowctl-cli/tests/surface.rs:290-304, high).

Migrated from `R-CLI-052` in `docs/spec/cli.md`, its second obligation.
