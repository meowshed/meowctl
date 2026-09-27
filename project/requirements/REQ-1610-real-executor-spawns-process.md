---
id: REQ-1610
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1610

`RealExecutor` MUST spawn the process.

`RealExecutor` is the implementation the binary constructs for a real run, so a
process is spawned there or nowhere, and `DryRunExecutor` and the scripted
executor stand in for it in a dry run and in tests (from
https://github.com/meowshed/meowctl/pull/58, high). No alternative was
considered, because it is the baseline (from
docs/design/0.2.0-requirement-tradeoffs.md:89, high).

Migrated from `R-EXEC-010` in `docs/spec/exec.md`.
