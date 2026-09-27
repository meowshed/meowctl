---
id: REQ-2203
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2203

`pkg(manager, name, version = "", **kwargs)` MUST record a package declaration.

The positional order is `manager` before `name`, so `pkg("brew", "git")`
installs git with Homebrew (from docs/spec/starlark.md, high). The order reads
backwards, and it is what `parsePkgArgs` in `internal/starlark/builtins.go`
accepts; most components write both arguments by keyword, which is why the order
rarely shows (from docs/spec/starlark.md, high). The alternative was a separate
builtin per manager, which lost because handlers are components and not code, so
the manager is an argument, and the table names no condition that would reverse
it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-003` in `docs/spec/starlark.md`, its first obligation.
