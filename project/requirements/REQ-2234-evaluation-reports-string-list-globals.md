---
id: REQ-2234
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2234

An evaluation MUST likewise report the value of every top-level name bound to a
list of strings.

`platforms` and `distros` are two, and they decide whether a component runs on
this machine; see REQ-3012 (from docs/spec/starlark.md, high). The alternative
was to read the guards by re-parsing the file, which lost because the evaluation
already has the values, and a second reader is a second grammar; revisit if
guards stop being top-level globals (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-034` in `docs/spec/starlark.md`.
