---
id: REQ-1703
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: judgement
verifier: person
---

# REQ-1703

`meowctl-exec` MUST NOT take a callback that suspends a renderer.

This is defect #11: in `v0.1.0` the hand-off is
`SuspendOutput func() (resume func())`, passed into `ctx.Capabilities` so that a
Starlark builtin can reach back into the terminal renderer (from
docs/spec/exec.md, high). The events replace it (from docs/spec/exec.md, high).
The alternative was to port the callback, which lost because it requires the
Starlark layer to hold a renderer concern (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The choice is argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

A reviewer of each change to `meowctl-exec` holds this, because a `Box<dyn
Fn()>` parameter needs no dependency a static check could see; the evidence that
would make it static is a test scanning `meowctl-exec`'s public signatures for
`Fn` parameters (from crates/meowctl-exec/Cargo.toml, high).

Migrated from `R-EXEC-022` in `docs/spec/exec.md`, its third obligation.
