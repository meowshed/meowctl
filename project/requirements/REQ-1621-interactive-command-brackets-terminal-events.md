---
id: REQ-1621
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1621

An interactive command MUST emit `TerminalRequested` before it starts and
`TerminalReleased` after it exits.

The sink stands down for the duration (from docs/spec/exec.md, high). The
alternative was to let the executor suspend the renderer, which lost because
that is `SuspendOutput`, which REQ-1622 forbids (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-EXEC-021` in `docs/spec/exec.md`.
