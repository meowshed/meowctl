---
id: REQ-3507
artifact: requirement
topic: cli
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3507

The version string MUST report the version, the target, the commit, and the
build date, in the form `internal/version/version.go` produces.

The version string format is part of the surface read from `internal/version/`
for parity with `v0.1.0` (from docs/spec/cli.md, high). The alternative was to
reproduce the `-ldflags` patching, which lost because a build outside the mise
task reports `dev` silently, and the trade-off record names no condition that
reverses it (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-013` in `docs/spec/cli.md`, its second obligation.
