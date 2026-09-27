---
id: REQ-2343
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2343

A `query_pm` call outside a hook MUST fail with a configuration error saying
that `query_pm` works only inside a hook.

Interrogating a package manager during discovery would run user code before the
graph exists, so the call can't work there, and the message has to tell the
author where it does.

Added during onboarding from the failure-path research; the shipped message is
`query_pm("brew") needs a package-manager registry and this evaluation has
none`, which blames the evaluation (from
crates/meowctl-starlark/src/evaluator.rs:62-75, high).
