---
id: REQ-1201
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1201

The component MUST own exactly these names, as `internal/cli/config.go` declares
them: `init.star`, `local.star`, `deps.mod`, `deps.lock`, `deps.local.mod`,
`deps.local.lock`, `state.toml`, `installed.lock`, `pkgs.lock`,
`pkgs.local.lock`, and `.hook-error`.

`.hook-error` was missing from an earlier draft of this list, which named the
files a run reads and writes and overlooked the one a shell spawn writes (from
docs/spec/config.md, high). The alternative was letting each crate name the file
it reads, which lost because `v0.1.0` does for `installed.lock` and `pkgs.lock`,
and both ended up defined inside `apply.go`, and the record expects nothing to
reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-001` in `docs/spec/config.md`.
