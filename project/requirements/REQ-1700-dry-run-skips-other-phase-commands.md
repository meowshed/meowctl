---
id: REQ-1700
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1700

`DryRunExecutor` MUST NOT run a command issued from any other phase.

The read-only phases are in REQ-1012, and the phase is what decides, because an
`install_check` hook that cannot ask `brew list` what is installed reports a
plan built on nothing (from docs/spec/exec.md, high). `validCheckPhases` in
`internal/ctx/methods.go` is the same set for the same reason (from
docs/spec/exec.md, high). The alternative was to refuse every subprocess in a
dry run, which lost because check hooks interrogate the system and refusing
makes their plan empty (from docs/design/0.2.0-requirement-tradeoffs.md, high).
Revisit it if a read-only phase is found to mutate (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The choice is argued at
length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-EXEC-011` in `docs/spec/exec.md`, its second obligation.
