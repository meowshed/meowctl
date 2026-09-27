---
id: REQ-2208
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2208

`select(cases)` MUST choose by platform using the matching rules in
`matchesPlatform` and `matchesLinuxDistro`, including matching a distribution
through `ID_LIKE` so a Mint machine matches a `debian` case.

The alternative was exact distribution matching only, which lost because a Mint
machine would miss every `debian` case, and `ID_LIKE` exists for this, and the
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-008` in `docs/spec/starlark.md`.
