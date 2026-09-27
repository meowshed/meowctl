---
id: REQ-2223
artifact: requirement
topic: starlark
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2223

The same module URL loaded twice within one command MUST be evaluated once and
cached, matching `v0.1.0`, so a component graph that loads a shared helper from
twenty components pays for it once.

The alternative was to re-evaluate per load, which lost because a graph loading
one helper from twenty components pays twenty times; revisit if a module needs
per-load state (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-023` in `docs/spec/starlark.md`.
