---
id: REQ-1432
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1432

A read of a file that does not exist MUST be distinguishable from a read that
failed for another reason.

Callers treat a missing lock file as a clean slate and a missing config file as
an error (from docs/spec/fs.md, high). The alternative was one error kind for
every read failure, which lost because callers treat a missing lock as a clean
slate and a missing config as fatal, and the record expects nothing to reverse
it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-032` in `docs/spec/fs.md`.
