---
id: REQ-2207
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2207

`platform()` MUST return a struct carrying the operating system, the
architecture, and, on Linux, the distribution and its `ID_LIKE` value.

`makePlatformStruct` in `v0.1.0` is the shape (from docs/spec/starlark.md,
high). The alternative was to return a dict, which lost because a struct is what
components index today; revisit if parity is abandoned (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-007` in `docs/spec/starlark.md`.
