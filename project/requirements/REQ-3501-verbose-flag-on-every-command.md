---
id: REQ-3501
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3501

`--verbose` MUST be available on every command.

`v0.1.0` declares `--verbose` as a global flag on the root command (from
https://github.com/meowshed/meowctl/issues/15 and
https://github.com/meowshed/meowctl/pull/16, medium). The confidence is medium
because `v0.1.0`'s `addLifecycleFlags` also declares it on the six lifecycle
commands (from https://github.com/meowshed/meowctl/pull/83, medium). A flag that
works on one command and not another is one nobody remembers (from
crates/meowctl-cli/tests/surface.rs:64-65, high).

Migrated from `R-CLI-004` in `docs/spec/cli.md`, its second obligation.
