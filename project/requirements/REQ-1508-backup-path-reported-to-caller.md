---
id: REQ-1508
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1508

When a backup is requested, the sibling path the file was renamed to MUST be
reported to the caller.

The alternative was silently overwriting a regular file, which lost because a
user loses a file they wrote by hand, with no record, and the record expects
nothing to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-021` in `docs/spec/fs.md`, its third obligation.
