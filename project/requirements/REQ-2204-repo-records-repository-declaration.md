---
id: REQ-2204
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2204

`repo(...)` MUST record a repository declaration targeting a named manager.

The alternative was to fold `repo` into `pkg`, which lost because a repository
is added once per manager and not once per package, and the table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). A repository declaration is
how a component adds a tap or a PPA before its packages resolve (from
crates/meowctl-starlark/tests/evaluation.rs:532-533, high).

Migrated from `R-STAR-004` in `docs/spec/starlark.md`.
