---
id: REQ-1003
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1003

A `ModuleRef` MUST distinguish a registry module, named by a bare identifier,
from a GitHub module, named `github:owner/repo@ref`.

The alternative was accepting any string as a `ModuleRef`, which lost because a
malformed ref becomes a registry lookup for a nonsense name and a confusing 404,
and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-003` in `docs/spec/common.md`, its first obligation.
