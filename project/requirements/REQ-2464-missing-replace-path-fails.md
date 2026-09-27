---
id: REQ-2464
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2464

A `replace` pointing at a path that does not exist MUST fail naming the path,
not fall back to fetching the original.

The alternative was to fall back to fetching the original, which lost because a
typo in a `replace` path would silently use upstream, which is the opposite of
what was asked, and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-064` in `docs/spec/module.md`.
