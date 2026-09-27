---
id: REQ-2509
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2509

Extraction MUST drop nothing when the entries are not all inside one top-level
directory.

The condition is exact and not a count of components, so the only module this
changes is one that ships one top-level directory and nothing beside it, and the
registry holds none (from docs/spec/module.md, high). The alternative was to
extract the archive as it comes, as `v0.1.0` does, which lost because every
entry of a GitHub archive is under `repo-<commit>/`, so every file of a
`github:` module sits one directory below where the loader looks; revisit if a
module ships one top-level directory on purpose (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-034` in `docs/spec/module.md`, its second obligation.
