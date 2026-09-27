---
id: REQ-1031
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1031

Every crate's error enum MUST be convertible to an exit code.

The alternative was letting crates choose codes, which lost because that is how
`v0.1.0` spreads `exitErrorf` through the CLI layer, and a crate that knows its
exit code knows about a process, and the record expects nothing to reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-031` in `docs/spec/common.md`, its first obligation.
