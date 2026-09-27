---
id: REQ-2233
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2233

An evaluation MUST report the value of every top-level name bound to a string.

`pm_name` is one, and it decides whether a component is a package-manager
handler and which manager it handles; see REQ-2601 (from docs/spec/starlark.md,
high). The evaluation reports a map in place of that one name, because the
evaluator has no business knowing which strings its callers care about (from
docs/spec/starlark.md, high). The alternative was to return `pm_name`
specifically, as `ScanGlobals` reads it, which lost because the evaluator would
then know which strings its callers care about, and the next caller would need
another method; revisit if nothing but `pm_name` is ever read this way (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-033` in `docs/spec/starlark.md`.
