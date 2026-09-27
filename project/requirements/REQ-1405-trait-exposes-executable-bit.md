---
id: REQ-1405
artifact: requirement
topic: fs
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1405

The trait MUST expose setting a file's executable bit, and reporting it.

This is the one exception [REQ-1404] allows, and it exists because a module
tarball ships scripts meowctl later executes; see [REQ-2432] (from
docs/spec/fs.md, high). It is the executable bit rather than a mode because that
is the whole of what the caller decides, since `v0.1.0` normalises every
extracted file to `0o644` or `0o755` and nothing asks for a third value (from
docs/spec/fs.md, high). The alternative was letting the module extractor `chmod`
directly, which lost because a path around the trait is a path a dry run can't
intercept, and the executable bit is the one mode a caller decides; revisit it
if modules stop shipping executables (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-FS-005` in `docs/spec/fs.md`, its first obligation.
