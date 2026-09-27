---
id: REQ-2831
artifact: requirement
topic: ctx
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2831

In a runtime hook phase, `ctx` MUST expose exactly the eight attributes
`ShellCtxAllowList` names: `emit`, `file_exists`, `list_dir`, `platform`,
`read_file`, `run`, `shell`, and `state_dir`.

A shell hook runs on every shell spawn and must have no persistent effect beyond
what it emits (from docs/spec/ctx.md, high). Every `shell` and `login` hook in
`meowctl-stdlib` and in `dotmeow` stays inside those eight, which was checked
rather than assumed (from docs/spec/ctx.md, high). The alternative was to give a
runtime hook the full `ctx`, which lost because it runs on every shell spawn,
and a write there runs thousands of times a day (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CTX-031` in `docs/spec/ctx.md`.
