---
id: REQ-2461
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2461

A module named in `deps.mod` but absent from the index MUST be reported by name,
distinguished from a version of it that does not exist.

The alternative was one "not found" for both, which lost because a module that
does not exist and a version that does not exist need different fixes, and the
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-061` in `docs/spec/module.md`.
