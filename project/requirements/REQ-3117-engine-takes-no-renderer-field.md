---
id: REQ-3117
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3117

The engine MUST NOT take a renderer as a field.

This is defect #11 (from docs/spec/engine.md, high). `Runner` in
`internal/lifecycle/runner.go` has a `Writer tui.Writer` field and falls back to
constructing one (from docs/spec/engine.md, high). The alternative was holding a
writer, as `Runner` does, which lost because that is defect #11, and
`SuspendOutput` is its cost (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-051` in `docs/spec/engine.md`, its second obligation.
