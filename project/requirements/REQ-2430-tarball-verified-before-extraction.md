---
id: REQ-2430
artifact: requirement
topic: module
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2430

Every fetched tarball MUST be verified against the integrity hash from the index
before it is extracted.

`v0.1.0` checks the tarball against the index hash when it downloads one and
from then on evaluates whatever is in the cache directory, so anything that can
write to `~/.cache` can change what a hook runs (from docs/spec/module.md,
high). The alternative was to verify the tarball only, which lost because
per-file hashes are what make a mutated cache detectable, and the table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-MODULE-030` in `docs/spec/module.md`, its first obligation.
