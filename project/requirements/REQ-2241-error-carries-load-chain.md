---
id: REQ-2241
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2241

An error raised inside a `load()`ed module MUST carry the chain of files that
led to it, not only the innermost one.

The alternative was to report the innermost file, which lost because a failure
in a shared stdlib helper would not say which component loaded it, and the table
names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-041` in `docs/spec/starlark.md`.
