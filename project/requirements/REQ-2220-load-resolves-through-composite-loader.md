---
id: REQ-2220
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2220

`load()` MUST resolve through a composite loader covering the four schemes
`internal/starlark/loader/composite.go` handles: a path relative to the config
directory, `self://`, `user://`, `github://owner/repo@ref//path`, and
`@name//path` for a registry module.

The alternative was one loader per scheme, chosen by the caller, which lost
because the caller would need to know the scheme before parsing the URL, and the
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-020` in `docs/spec/starlark.md`.
