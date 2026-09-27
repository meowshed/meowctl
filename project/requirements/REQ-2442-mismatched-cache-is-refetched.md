---
id: REQ-2442
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2442

A cached module whose files do not match the recorded hashes MUST be re-fetched
rather than used, because the cache is not authoritative.

The alternative was to treat a mismatch as an error, which lost because the
common cause is a truncated cache from a killed process and not tampering;
revisit if tampering becomes a realistic threat here (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The table marks this row as
argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-042` in `docs/spec/module.md`, its first obligation.
