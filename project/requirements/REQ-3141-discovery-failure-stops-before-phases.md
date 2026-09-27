---
id: REQ-3141
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3141

A failure during discovery MUST stop the command before any phase runs.

The graph and the package-manager registry need every component, so a run
without one of them would order or dispatch wrongly.

Added during onboarding from the failure-path research; the shipped `discover`
stops at the first error, and nothing runs (from
crates/meowctl-engine/src/discovery.rs:205-265, high).
