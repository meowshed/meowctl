---
id: REQ-1420
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1420

Creating a symlink over an existing symlink MUST replace it.

`internal/ctx/methods.go` reads the prior target before replacing, and
[REQ-2014] depends on getting it (from docs/spec/fs.md, high). The alternative
was failing on an existing symlink, which lost because re-running `apply` is the
normal case and would fail every time, and the record expects nothing to reverse
it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-020` in `docs/spec/fs.md`, its first obligation.
