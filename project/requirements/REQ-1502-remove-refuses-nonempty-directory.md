---
id: REQ-1502
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1502

`remove` MUST refuse a directory that is not empty.

The recursive method exists for the module cache, which replaces a module's
directory when it no longer matches what was recorded; see [REQ-2442] (from
docs/spec/fs.md, high). `v0.1.0` has no equivalent, because nothing there
removes a cache entry, and a module whose files changed under it is evaluated as
it stands (from docs/spec/fs.md, high). The alternative was one `remove` that
takes a directory too, or recursion at the call site, which lost because an undo
removing one empty directory and a cache discarding a subtree are different
decisions, and a single method makes the dangerous one the default; revisit it
if nothing needs to discard a subtree (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-006` in `docs/spec/fs.md`, its second obligation.
