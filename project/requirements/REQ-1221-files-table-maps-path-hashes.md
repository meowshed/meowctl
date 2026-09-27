---
id: REQ-1221
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1221

`files`, when present, MUST map each extracted path, relative to the module
root, to its own integrity hash.

Nothing writes it, because `v0.1.0` declares the table and never fills it, and
`v0.2.0` records the per-file hashes in the cache instead, for the reasons in
[REQ-2444] (from docs/spec/config.md, high). The schema keeps it because a lock
either binary wrote has to round-trip through the other (from
docs/spec/config.md, high). The alternative was hashing the tarball only, which
lost because a cache mutated after extraction would verify, and the record
expects nothing to reverse it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

Migrated from `R-CONFIG-021` in `docs/spec/config.md`.
