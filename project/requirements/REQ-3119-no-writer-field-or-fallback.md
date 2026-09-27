---
id: REQ-3119
artifact: requirement
topic: engine
class: functional
status: withdrawn
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3119

Replaced by REQ-3051 and REQ-3117.

Neither a writer field nor a fallback that constructs a writer MUST exist in the
engine.

Withdrawn during onboarding: the source's "neither MUST exist here" restates the
first two obligations of `R-ENGINE-051` against `v0.1.0`'s `Runner`. A writer
field is what [REQ-3117] forbids, and a fallback that constructs a writer is a
way of holding one, which [REQ-3051] forbids, so no test or check could hold
this apart from those two (from docs/spec/engine.md:199-202, high).

This is defect #11 (from docs/spec/engine.md, high). `Runner` in
`internal/lifecycle/runner.go` has a `Writer tui.Writer` field and falls back to
constructing one (from docs/spec/engine.md, high). The alternative was holding a
writer, as `Runner` does, which lost because that is defect #11, and
`SuspendOutput` is its cost (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-051` in `docs/spec/engine.md`, its fourth obligation.
