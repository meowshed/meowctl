---
id: REQ-1604
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1604

A non-zero exit MUST NOT be an error.

It is a result the caller inspects (from docs/spec/exec.md, high). An error is
reserved for a command that could not be started or whose output could not be
read (from docs/spec/exec.md, high). The alternative was to treat a non-zero
exit as an error, which lost because every interrogation hook would have to
catch it, and `brew list` exiting 1 is information (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-EXEC-004` in `docs/spec/exec.md`.
