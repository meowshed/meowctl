---
id: REQ-2205
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2205

`query_pm(manager)` MUST call the registered handler's `interrogate` on its own
evaluation and return what it returns.

It is the one builtin that runs user code during evaluation; see REQ-2620 (from
docs/spec/starlark.md, high). The alternative was to return a cached
interrogation, which lost because a hook asks because it wants the current
state; revisit if interrogation becomes slow enough to need caching within one
run (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-005` in `docs/spec/starlark.md`.
