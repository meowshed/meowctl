---
id: REQ-2444
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2444

The cache record MUST be written inside the cache directory under a name that
cannot be loaded as a module file.

The alternative was to put the per-file hashes in the `files` table of
`deps.lock`, which lost because `v0.1.0` never writes that table and replaces an
entry wholesale, so the hashes would vanish on its next run, and filling it ends
the byte-identical lock; revisit if `v0.1.0` is no longer installed anywhere
(from docs/design/0.2.0-requirement-tradeoffs.md, high). The table marks this
row as argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-044` in `docs/spec/module.md`, its first obligation.
