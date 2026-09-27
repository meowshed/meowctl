---
id: REQ-1605
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1605

The environment MUST be the parent environment merged with per-call overrides,
with the override winning.

`mergeRunEnv` in `internal/ctx/methods.go` is the behaviour (from
docs/spec/exec.md, high). The alternative was to replace the environment, which
lost because a hook would lose `PATH` and every tool with it (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-EXEC-005` in `docs/spec/exec.md`.
