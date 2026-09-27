---
id: REQ-3030
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3030

A phase MUST run its hook for every component in order.

An earlier draft collected every failure and reported them together, and
`fix(lifecycle): fail-fast on component failure and trigger rollback` changed
that, because a component failing early makes every component that depended on
it fail too -- fifty tools reporting "mise: executable file not found" when mise
is what failed (from docs/spec/engine.md, high). `PhaseError` still carries a
list because one failure is a list of one (from docs/spec/engine.md, high). The
alternative was collecting every failure and reporting them together, which lost
because one component failing makes everything after it fail too, and fifty
errors about a tool that never installed bury the one that matters (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a phase appears
whose components are genuinely independent (from
docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-030` in `docs/spec/engine.md`, its first obligation.
