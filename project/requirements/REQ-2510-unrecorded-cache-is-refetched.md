---
id: REQ-2510
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2510

A cache with no record at all MUST be treated the same way as one whose files do
not match: the module was extracted by something that did not write one, and
what it holds is unknown.

A cache populated by `v0.1.0` has no per-file record, so meowctl re-fetches it
once (from docs/spec/module.md, high). The alternative was to treat a mismatch
as an error, which lost because the common cause is a truncated cache from a
killed process and not tampering; revisit if tampering becomes a realistic
threat here (from docs/design/0.2.0-requirement-tradeoffs.md, high). The table
marks this row as argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-042` in `docs/spec/module.md`, its second obligation.
