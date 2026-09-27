---
id: REQ-2621
artifact: requirement
topic: pm
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2621

`interrogate` MUST be callable during a dry run, because it reads rather than
writes; see REQ-1611.

The alternative was to skip interrogation in a dry run, which lost because every
check hook would report nothing (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names
nothing that would reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-PM-021` in `docs/spec/pm.md`.
