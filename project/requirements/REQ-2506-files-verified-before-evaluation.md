---
id: REQ-2506
artifact: requirement
topic: module
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2506

Every extracted file MUST be verified against its recorded per-file hash before
it is evaluated.

`v0.1.0` checks the tarball against the index hash when it downloads one and
from then on evaluates whatever is in the cache directory, so anything that can
write to `~/.cache` can change what a hook runs (from docs/spec/module.md,
high). The per-file hashes are recorded in the cache beside the extracted files,
because `v0.1.0` never writes the `files` table of `deps.lock` and its writer
replaces a whole entry, so filling that table would break the byte-identical
lock REQ-1223 requires and lose the hashes the first time a `v0.1.0` binary
rewrote the entry (from docs/spec/module.md, high). The hashes say whether the
cache still holds what was extracted, and that question is answerable where the
cache is (from docs/spec/module.md, high). The alternative was to verify the
tarball only, which lost because per-file hashes are what make a mutated cache
detectable, and the table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-030` in `docs/spec/module.md`, its second obligation.
