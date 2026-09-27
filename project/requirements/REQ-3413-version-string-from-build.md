---
id: REQ-3413
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3413

The version string MUST come from the build rather than from variables patched
by linker flags.

The alternative was to reproduce the `-ldflags` patching, which lost because a
build outside the mise task reports `dev` silently, and the trade-off record
names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-013` in `docs/spec/cli.md`, its first obligation.
