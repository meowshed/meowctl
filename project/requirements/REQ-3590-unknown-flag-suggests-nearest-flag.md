---
id: REQ-3590
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3590

An unknown flag MUST suggest the nearest flag when one is close.

A mistyped flag is corrected the same way as a mistyped command [REQ-3518]
(inferred during onboarding, low). No test holds it yet, because
`crates/meowctl-cli/tests/surface.rs:299-304` checks only that an unknown flag
is a usage error; the evidence still missing is a test there asserting the
suggestion (from crates/meowctl-cli/tests/surface.rs:299-304, high).

Added during onboarding from the research on thin requirements, which split it
from the flag half of `R-CLI-052`; the shipped code keeps clap's default
`suggestions` feature (from Cargo.toml:45, high).
