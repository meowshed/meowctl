---
id: REQ-2810
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2810

`ctx` MUST expose exactly the twenty-four methods `New` registers, under those
names: `log`, `env`, `write_file`, `append_file`, `delete_file`, `copy_file`,
`symlink`, `remove_symlink`, `link_file`, `mkdir`, `read_file`, `file_exists`,
`list_dir`, `run`, `git_clone`, `download`, `defaults_write`, `plist_set`,
`prompt`, `emit`, `add_path`, `render`, `render_file`, and `which`.

The surface is frozen at what `v0.1.0` exposes, and a component written for
`v0.1.0` must run unchanged (from docs/spec/ctx.md, high). The alternative was
to add a method that would be useful, which lost for the reason REQ-2201 gives:
a component using it stops working on `v0.1.0` (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it once the Go tree
is deleted, a condition the cutover has met without reversing the requirement
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CTX-010` in `docs/spec/ctx.md`.
