---
id: REQ-1211
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1211

`deps.mod` MUST support the statements `internal/modfile/modfile.go` documents:
`module(name, version)`, `dep(name, version)` for a registry dependency,
`dep(name, source)` for a GitHub dependency, and `replace(name, path)` or
`replace(name, source)`.

A module's own `MODULE.meow` uses the same statements plus `compat` on
`module()`; see [REQ-2206] (from docs/spec/config.md, high). Meowctl writes
`deps.mod` and never writes a `MODULE.meow`, so this component's writer emits no
`compat` (from docs/spec/config.md, high). The alternative was a TOML manifest,
which lost because `deps.mod` is Starlark today and users edit it; revisit it if
parity is abandoned (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-011` in `docs/spec/config.md`.
