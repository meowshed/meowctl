---
id: REQ-1403
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1403

A write MUST be atomic: the implementation writes a temporary file in the
destination's directory and renames it into place, so a reader never observes a
partial file and a crash mid-write leaves the prior content.

`internal/lock/write.go` and `internal/state/state.go` both do this, and the
lock files depend on it (from docs/spec/fs.md, high). The alternative was
writing in place, which lost because a reader sees a partial lock file and a
crash loses the previous one, and the record says it never reverses, because the
lock files depend on it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-003` in `docs/spec/fs.md`.
