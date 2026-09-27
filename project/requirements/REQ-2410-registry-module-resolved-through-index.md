---
id: REQ-2410
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2410

A registry module MUST be resolved through the registry index, which lists each
module's versions, tarball URLs, and integrity hashes.

The alternative was to fetch modules by URL only, which lost because the index
is what makes MVS possible, since it lists the versions; revisit if registry
modules are dropped (from docs/design/0.2.0-requirement-tradeoffs.md, high). A
sync whose every dependency is local or on GitHub doesn't fetch the index,
because a machine with no network still has to be able to run (from
crates/meowctl-module/tests/syncing.rs:577-579, high).

Migrated from `R-MODULE-010` in `docs/spec/module.md`.
