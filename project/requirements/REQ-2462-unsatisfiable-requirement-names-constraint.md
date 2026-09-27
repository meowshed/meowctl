---
id: REQ-2462
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2462

A requirement no version satisfies MUST name the module and the constraint.

The alternative was to report "resolution failed", which lost because the user
has to bisect their manifests to find which constraint, and the table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-062` in `docs/spec/module.md`.
