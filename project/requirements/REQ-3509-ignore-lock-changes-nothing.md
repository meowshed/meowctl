---
id: REQ-3509
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3509

`--ignore-lock` MUST change nothing, which is what `v0.1.0` does.

`v0.1.0` declares the flag in `internal/cli/lifecycle.go:110` and reads it
nowhere: `IgnoreLock` is set on `runConfig` and no code path consults it (from
docs/spec/cli.md, high). Implementing it would be new behaviour and would need
its own requirement (from docs/spec/cli.md, high). The alternative was to drop
the flag or implement it, which lost because dropping it breaks a script that
passes it and implementing it is behaviour neither binary has ever had; revisit
if somebody needs to resolve without the lock (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-016` in `docs/spec/cli.md`, its second obligation.
