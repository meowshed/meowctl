---
id: REQ-3060
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3060

A hook failing MUST name the component, the phase, and the underlying error.

The alternative was reporting the underlying error alone, which lost because the
user cannot tell which component or phase produced it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-060` in `docs/spec/engine.md`.
