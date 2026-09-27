---
id: REQ-2015
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2015

`LinkFile`'s inverse MUST remove the symlink.

A backup that is not restored is how a user loses a file they had before meowctl
ran (from docs/spec/ops.md, high). The alternative was to remove the link and
stop, which lost because the backup is the user's original file, and not
restoring it is data loss (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names nothing that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-015` in `docs/spec/ops.md`, its first obligation.
