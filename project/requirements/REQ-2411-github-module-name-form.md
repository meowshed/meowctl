---
id: REQ-2411
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2411

A GitHub module MUST be named `github:owner/repo@ref`, where the ref is a tag or
a branch.

The alternative was to pin to the ref, which lost because a branch moves, and
two machines get different code from one lock, and the table names no condition
that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-011` in `docs/spec/module.md`, its first obligation.
