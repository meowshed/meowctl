---
id: REQ-2032
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2032

When the journal append fails, the operation MUST NOT be applied.

An effect applied with no record of how to undo it is the state this component
exists to prevent (from docs/spec/ops.md, high). The alternative was to apply
anyway and warn, which lost for that reason (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-OPS-032` in `docs/spec/ops.md`.

Reworded during onboarding from the failure-path research to state the effect,
not the error; the shipped `Core::apply` appends the inverse and returns on any
error before `op.apply`, and a test checks it (from
crates/meowctl-ctx/src/state.rs:142-164 and
crates/meowctl-ctx/tests/surface.rs:694, high).
