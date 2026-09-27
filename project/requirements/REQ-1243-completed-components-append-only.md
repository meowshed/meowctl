---
id: REQ-1243
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1243

`completed_components` MUST be append-only within a run.

The alternative was overwriting it per run, which lost because `status` shows
history, and resuming an interrupted run needs the records; revisit it if the
ledger grows enough to need pruning, which `--all` already anticipates (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-043` in `docs/spec/config.md`, its first obligation.
