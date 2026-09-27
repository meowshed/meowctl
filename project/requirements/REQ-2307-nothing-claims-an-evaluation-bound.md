---
id: REQ-2307
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-2307

meowctl MUST NOT claim to bound a configuration that loops forever from inside
the evaluation.

This replaces a requirement that claimed `v0.1.0` inherits the `go.starlark.net`
step limit, which is off unless `SetMaxExecutionSteps` is called and `v0.1.0`
never calls it (from docs/spec/starlark.md, high). A spec that promised a bound
would be promising something no reader could rely on (from
docs/spec/starlark.md, high). The alternative was to claim a bound neither
binary has, which lost because `go.starlark.net`'s step limit is off unless set,
`v0.1.0` never sets it, and `starlark-rust` has no supported hook at all;
revisit if `starlark-rust` gains one (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-052` in `docs/spec/starlark.md`, its second obligation.

Onboarding reworded the source's "No ... MUST" to MUST NOT, because the record's
check refuses a negated subject; the meaning is unchanged.
