---
id: REQ-2512
artifact: requirement
topic: module
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: judgement
verifier: person
---

# REQ-2512

`v0.1.0` writes half of the cache record already, as the `.sri` sidecar that
carries the tarball hash, and the format here MUST keep the `.sri` sidecar
readable by `v0.1.0`: the two binaries share one cache.

The per-file hashes go in a second file beside the sidecar, which `v0.1.0`
ignores (from docs/spec/module.md, high). The alternative was to put the
per-file hashes in the `files` table of `deps.lock`, which lost because `v0.1.0`
never writes that table and replaces an entry wholesale, so the hashes would
vanish on its next run, and filling it ends the byte-identical lock; revisit if
`v0.1.0` is no longer installed anywhere (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The table marks this row as
argued at length in `docs/design/0.2.0-decisions.md` (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

A reviewer holds this by comparing the sidecar `v0.2.0` writes against the
format `v0.1.0` recorded, because nothing can run `v0.1.0` since the Go tree and
the compatibility corpus were deleted (from CLAUDE.md parity_is_the_contract,
high).

Migrated from `R-MODULE-044` in `docs/spec/module.md`, its third obligation.
