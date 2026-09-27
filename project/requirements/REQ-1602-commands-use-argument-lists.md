---
id: REQ-1602
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1602

A command MUST be built from a program name and an argument list, never from a
shell string.

`v0.1.0` does the same, and it is what keeps a component's arguments from being
reinterpreted by a shell that the component author did not know was there (from
docs/spec/exec.md, high). The alternative was to accept a shell string, which
lost because a component's arguments would get reinterpreted by a shell its
author did not know was there (from docs/design/0.2.0-requirement-tradeoffs.md,
high). Revisit it if a component needs shell syntax, which it can get today by
invoking a shell explicitly (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-EXEC-002` in `docs/spec/exec.md`.
