---
id: REQ-1263
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1263

Two concurrent runs writing the same file MUST NOT interleave into a corrupt
result.

The atomic rename gives this, and nothing depends on a lock (from
docs/spec/config.md, high). The alternative was taking a lock file, which lost
because the atomic rename already gives it, and a lock adds a stale-lock failure
mode; revisit it if two runs need to coordinate beyond file writes (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-063` in `docs/spec/config.md`.
