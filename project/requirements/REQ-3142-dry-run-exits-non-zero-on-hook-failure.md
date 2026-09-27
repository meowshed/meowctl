---
id: REQ-3142
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3142

A dry run MUST exit non-zero when a hook fails against the dry-run effects.

A dry run that shows success for a hook that will fail is the silent false
success ADR-0023 calls worse than no dry run.

A dry run runs hooks against the dry-run effects, as REQ-3172 requires, so a
hook that fails there fails the dry run.

Added during onboarding from the failure-path research; the shipped
`apply --dry-run` renders the plan and runs no hook (from
crates/meowctl-cli/src/run/commands.rs:447-455, high).
