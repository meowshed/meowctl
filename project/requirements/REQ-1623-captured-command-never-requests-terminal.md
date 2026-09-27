---
id: REQ-1623
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1623

A command that is not interactive MUST NOT request the terminal.

Its output is captured and rendered under its component (from docs/spec/exec.md,
high). The alternative was to request the terminal for every command, which lost
because every captured command would tear down the live region (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-EXEC-023` in `docs/spec/exec.md`.
