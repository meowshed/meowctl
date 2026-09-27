---
id: REQ-2451
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2451

Syncing MUST write the full resolution: version, source, integrity, and commit
SHA where applicable.

The per-file hashes go to the cache record and not to the lock; REQ-2430 says
why (from docs/spec/module.md, high). The alternative was to record the version
only, which lost because a cache could not be verified and a GitHub ref could
not be pinned, and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-051` in `docs/spec/module.md`.
