---
id: REQ-1507
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1507

When a backup is requested, the file MUST be renamed to a sibling path before
the symlink is created.

The alternative was silently overwriting a regular file, which lost because a
user loses a file they wrote by hand, with no record, and the record expects
nothing to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-021` in `docs/spec/fs.md`, its second obligation.
