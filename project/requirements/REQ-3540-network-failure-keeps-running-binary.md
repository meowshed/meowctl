---
id: REQ-3540
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3540

A network failure during `self-update` MUST leave the running binary unchanged.

A user whose update failed still needs a working `meowctl` to retry.

Added during onboarding from the failure-path research; the shipped command
writes nothing until all three requests succeed and the checksum matches (from
crates/meowctl-cli/src/update.rs:33-77, high).
