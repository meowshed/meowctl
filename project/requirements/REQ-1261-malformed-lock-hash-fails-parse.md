---
id: REQ-1261
artifact: requirement
topic: config
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1261

A lock file that names a module with an integrity hash in a form [REQ-1004]
rejects MUST fail to parse rather than be treated as unverified.

The alternative was accepting an unparseable hash as unverified, which lost
because a module would load without verification and nothing would say so, and
the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-061` in `docs/spec/config.md`.
