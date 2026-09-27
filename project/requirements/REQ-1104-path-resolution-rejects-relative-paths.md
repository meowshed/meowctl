---
id: REQ-1104
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1104

Path resolution MUST reject a path that is not absolute once expanded.

`requirePath` rejects only the empty string, so `v0.1.0` accepts a relative path
and resolves it against whatever the process working directory happens to be
(from docs/spec/common.md, high). A hook has no defined working directory, so
that result is unpredictable, and refusing is a deliberate change (from
docs/spec/common.md, high). `~user` is refused for the same reason, because
`expandPath` passes it through, which creates a directory literally named
`~user` (from docs/spec/common.md, high). The alternative was accepting relative
paths and `~user`, as `v0.1.0` does, which lost because a hook has no defined
working directory, so a relative path resolves somewhere its author can't
predict, and `~user` becomes a directory literally named `~user`; revisit it if
a hook gains a defined working directory (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-022` in `docs/spec/common.md`, its second obligation.
