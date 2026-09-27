---
id: REQ-1233
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1233

A module's fingerprint MUST be its resolved semver version for a registry
module, its commit SHA for a GitHub module, and its integrity hash when neither
is available. `moduleFingerprint` is the fallback chain.

The chain exists so that a module bumped without a version change still
invalidates (from docs/spec/config.md, high). The alternative was using the
version alone, which lost because a GitHub module re-synced to a new commit
keeps its version and wouldn't invalidate, and the record expects nothing to
reverse it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-033` in `docs/spec/config.md`.
