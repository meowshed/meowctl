---
id: REQ-1422
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1422

Removing a symlink MUST fail when the path is not a symlink, so that a mistaken
`remove_symlink` on a real file cannot delete it.

A mistaken `remove_symlink` on a real file can't delete it (from
docs/spec/fs.md, high). The alternative was removing whatever is at the path,
which lost because `remove_symlink` on a mistyped path would delete a real file,
and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-022` in `docs/spec/fs.md`.
