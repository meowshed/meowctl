---
id: REQ-3106
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3106

A name in a filter that matches nothing MUST be an error rather than an empty
run.

This changes behaviour on purpose: `filterDecls` in `v0.1.0` returns an error
already, and the requirement states it as an obligation rather than an
implementation detail (from docs/spec/engine.md, high). The alternative was
running nothing when the filter matches nothing, which lost because
`meowctl apply nvim` for a component called `neovim` would report success and do
nothing (from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if
scripts need the permissive behaviour often enough to complain (from
docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-016` in `docs/spec/engine.md`, its second obligation.
