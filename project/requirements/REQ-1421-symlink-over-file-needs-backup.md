---
id: REQ-1421
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1421

Creating a symlink over an existing regular file MUST fail unless the caller
asked for a backup.

The alternative was silently overwriting a regular file, which lost because a
user loses a file they wrote by hand, with no record, and the record expects
nothing to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-021` in `docs/spec/fs.md`, its first obligation.
