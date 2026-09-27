---
id: REQ-2828
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2828

`append_file(dst, content, marker = None)` MUST accept a caller-supplied marker.

A component that re-runs with the same marker replaces its own block rather than
appending a second copy, which is why the argument exists; see REQ-2011 (from
docs/spec/ctx.md, high). The alternative was to always generate the marker,
which lost because a component re-running would append a second block in place
of replacing its own (from docs/design/0.2.0-requirement-tradeoffs.md, high).
The trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-028` in `docs/spec/ctx.md`, its first obligation.
