---
id: REQ-1704
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-1704

The workspace MUST NOT define a type, function, field or trait named
`SuspendOutput`.

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

The source said "anywhere in the tree", which `rg SuspendOutput` fails on the
doc comments at `crates/meowctl-engine/src/runner.rs:9`,
`crates/meowctl-engine/src/lib.rs:6` and `crates/meowctl-exec/src/lib.rs:8` that
explain its absence; the evidence still missing is a scanning test that matches
definitions and skips comments (from crates/meowctl-engine/src/runner.rs:9,
high).

Migrated from `R-EXEC-022` in `docs/spec/exec.md`, its fourth obligation.
