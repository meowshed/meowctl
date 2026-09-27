---
id: REQ-1630
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1630

A command that does not exist MUST produce a distinct error naming it, not a
generic spawn failure.

A hook that calls a package manager that is not installed is the common case
(from docs/spec/exec.md, high). The alternative was a generic spawn failure,
which lost because it says "no such file or directory" without the command name,
for the commonest failure there is (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-EXEC-030` in `docs/spec/exec.md`.
