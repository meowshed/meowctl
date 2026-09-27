---
id: REQ-1044
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1044

The `Event` set MUST include `ShellLine` and `PathPrepended`, which `ctx.emit`
and `ctx.add_path` produce.

Both are effects on the process rather than on the filesystem, and `ctx` reaches
every effect through something it was given; see [REQ-2824] and [REQ-2825] (from
docs/spec/common.md, high). `v0.1.0` writes to stdout and calls `setenv` from
inside the method, which is why neither can be dry-run, rendered as JSON, or
tested without a process (from docs/spec/common.md, high). The alternative was
letting `ctx` write to stdout and call `setenv`, as `v0.1.0` does, which lost
because neither can then be dry-run, rendered as JSON, or tested without a
process, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-044` in `docs/spec/common.md`.
