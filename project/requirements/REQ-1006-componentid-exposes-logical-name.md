---
id: REQ-1006
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1006

A `ComponentId` MUST expose its logical name: the last path segment of a
qualified form, and the whole of a bare one.

So `@stdlib//components/node` and `github://o/r//components/node` are both
`node` (from docs/spec/common.md, high). This is the name an `after` list refers
to and the key a lock entry and a sentinel record use, and `logicalName` in
`internal/starlark/accumulator.go` is the behaviour (from docs/spec/common.md,
high). The alternative was computing the logical name at each call site, which
lost because `v0.1.0` has one helper and four places that re-derive it, each a
place to get `//` splitting wrong, and the record expects nothing to reverse it
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-006` in `docs/spec/common.md`.
