---
id: REQ-1101
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1101

Parsing a string that is neither a registry module nor a GitHub module as a
`ModuleRef` MUST fail rather than produce a registry module with a strange name.

The alternative was accepting any string as a `ModuleRef`, which lost because a
malformed ref becomes a registry lookup for a nonsense name and a confusing 404,
and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-003` in `docs/spec/common.md`, its second obligation.
