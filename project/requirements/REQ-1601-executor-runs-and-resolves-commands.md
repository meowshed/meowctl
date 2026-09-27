---
id: REQ-1601
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1601

`Executor` MUST cover running a command to completion and resolving a command
name on `PATH`.

Nothing else spawns a process (from docs/spec/exec.md, high). The alternative
was to let crates spawn processes directly, which lost because dry-run and test
injection would then have no single place to live (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-EXEC-001` in `docs/spec/exec.md`.
