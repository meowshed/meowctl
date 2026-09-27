---
id: REQ-3541
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3541

A failed `init <repo-url>` MUST leave the configuration directory as it was
before the command.

Half a configuration is worse than none, because the next `apply` runs it, and
with `--force` the leftovers merge into the old tree.

Added during onboarding from the failure-path research; the shipped command
writes nothing when the fetch fails, but writes the archive straight into the
root, so a later failure leaves files behind (from
crates/meowctl-cli/src/run/writing.rs:260-315 and
crates/meowctl-module/src/archive.rs:115-128, high).
