---
id: REQ-3403
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3403

`init` with no argument MUST scaffold a config directory.

The alternative was separate `init` and `bootstrap` commands, which lost because
`v0.1.0` merged them on purpose and splitting them adds a command back; revisit
if parity is abandoned (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-003` in `docs/spec/cli.md`, its first obligation.
