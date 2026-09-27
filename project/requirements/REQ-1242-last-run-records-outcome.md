---
id: REQ-1242
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1242

`last_run` MUST record the phase set, the start time in UTC, whether the run
completed, and the rollback outcome.

The alternative was recording less, which lost because `status` exists to report
exactly these four, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-042` in `docs/spec/config.md`.
