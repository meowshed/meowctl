---
id: REQ-1202
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1202

Every write MUST be atomic through `FileSystem`; see [REQ-1403].

These files are read by the next run and by a second binary, and a partial one
is indistinguishable from a valid one that lost data (from docs/spec/config.md,
high). `v0.1.0` is atomic for `deps.lock` and `state.toml` and isn't for
`deps.mod`, which `internal/modfile/modfile.go` writes with a plain
`os.WriteFile` (from docs/spec/config.md, high). Making all of them atomic is a
deliberate change, because a `deps.mod` truncated by a crash during
`meowctl dep add` leaves a configuration that doesn't parse (from
docs/spec/config.md, high). The alternative was writing `deps.mod` with a plain
write, as `modfile.Write` does, which lost because a crash during
`meowctl dep add` leaves a `deps.mod` that doesn't parse, and the record expects
nothing to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-002` in `docs/spec/config.md`.
