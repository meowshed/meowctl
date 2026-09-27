---
id: REQ-2340
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2340

A `select` call whose cases match nothing and that carries no
`//conditions:default` MUST fail the evaluation.

A configuration meant to cover this machine that doesn't should say so at the
call, because returning `None` passes a silent gap into a hook.

Added during onboarding from the failure-path research; the shipped builtin
fails with `select: no condition matched and no //conditions:default provided`,
and no test covers it (from crates/meowctl-starlark/src/builtins.rs:275-303,
high).
