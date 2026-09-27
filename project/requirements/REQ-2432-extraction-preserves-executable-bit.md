---
id: REQ-2432
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2432

Extraction MUST preserve the executable bit from the tar entry, deriving the
on-disk mode as `0o644` or `0o755`.

`v0.1.0` fixed this in `fix: correct module updates`, and getting it wrong
breaks every component that executes a file it shipped (from
docs/spec/module.md, high). The alternative was a fixed mode, which lost because
every component shipping a script would break, and `v0.1.0` fixed this once
already, and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-032` in `docs/spec/module.md`.
