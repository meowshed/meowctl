---
id: REQ-2402
artifact: requirement
topic: module
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2402

Versions MUST be compared by semver ordering, with the literal `none` ordering
below every valid version.

The alternative was lexicographic ordering, which lost because `v0.10.0` would
sort below `v0.9.0`, and the table names no condition that would reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high). Lexicographic ordering
is how a resolution silently picks an older module, and `none` ordering lowest
is how a module with no requirement drops out of a build list (from
crates/meowctl-module/src/version.rs:132-133 and :150-151, high).

Migrated from `R-MODULE-002` in `docs/spec/module.md`.
