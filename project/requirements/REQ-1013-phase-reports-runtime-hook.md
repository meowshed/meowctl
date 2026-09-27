---
id: REQ-1013
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1013

`Phase` MUST report whether it is a runtime hook phase. The runtime hook phases
are `shell` and `login`.

Only in those phases does `ctx.emit` write to stdout; see [REQ-2824] (from
docs/spec/common.md, high). The alternative was treating `emit` as always on,
which lost because emitting outside a hook phase writes into a piped run's
stdout; revisit it if `emit` gets its own channel (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-013` in `docs/spec/common.md`.
