---
id: REQ-1232
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1232

Writing `installed.lock` MUST always produce schema version 2, sorted by
component name, so the file does not churn between runs.

A sorted file doesn't churn between runs (from docs/spec/config.md, high). The
alternative was preserving input order, which lost because the file churns
between runs and every apply shows a diff, and the record expects nothing to
reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-032` in `docs/spec/config.md`.
