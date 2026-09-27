---
id: REQ-2232
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2232

A global that is present but not callable MUST be an error naming the component
and the hook.

The alternative was to ignore a non-callable global, which lost because a typo
that shadows a hook name would silently skip the hook, and the table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-032` in `docs/spec/starlark.md`.
