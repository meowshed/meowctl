---
id: REQ-2450
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2450

A module already present in the lock MUST be used at the locked version without
running selection again. Selection runs when a module is absent, or when the
caller asked to ignore the lock.

The alternative was to re-resolve every run, which lost because resolution would
vary with what the registry published today, and the table names no condition
that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-050` in `docs/spec/module.md`.
