---
id: REQ-3464
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3464

`status` and `doctor` MUST report the flag when it is present.

The flag is `.hook-error` (from docs/spec/cli.md, high). The alternative was to
say nothing and let the user find the file, which lost because a shell silently
missing its integration is exactly what nobody thinks to look for, and the
trade-off record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-064` in `docs/spec/cli.md`, its first obligation.
