---
id: REQ-2511
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2511

The cache record MUST cover the tarball's integrity as well as the per-file
hashes.

`v0.1.0` writes half of the record already, as the `.sri` sidecar that carries
the tarball hash (from docs/spec/module.md, high). The alternative was to put
the per-file hashes in the `files` table of `deps.lock`, which lost because
`v0.1.0` never writes that table and replaces an entry wholesale, so the hashes
would vanish on its next run, and filling it ends the byte-identical lock;
revisit if `v0.1.0` is no longer installed anywhere (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The table marks this row as
argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-044` in `docs/spec/module.md`, its second obligation.
