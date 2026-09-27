---
id: REQ-1620
artifact: requirement
topic: exec
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1620

A process's output MUST reach the caller as `ProcessOutput` events carrying the
stream and the line, in addition to being accumulated for the result.

A sink decides how to show it; see REQ-3221 (from docs/spec/exec.md, high). The
alternative was to buffer output and hand it back at the end, which lost because
a long install shows nothing until it finishes (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-EXEC-020` in `docs/spec/exec.md`.
