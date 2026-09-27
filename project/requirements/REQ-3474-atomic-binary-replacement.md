---
id: REQ-3474
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3474

Replacing the running binary MUST be atomic: written beside it, made executable,
then renamed over it.

A partial write over the binary leaves a machine with no working `meowctl` and
no way to fetch one (from docs/spec/cli.md, high). The alternative was to write
straight over the running binary, which lost for that reason, and the trade-off
record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Every test runs the path where each step succeeds, and none covers the branch at
`crates/meowctl-cli/src/update.rs:109-114`; the evidence still missing is two
tests in that module's test module that fail the write and then the rename and
assert that the old binary survives and no `.new` file is left (from
crates/meowctl-cli/src/update.rs:138-200, high).

Migrated from `R-CLI-074` in `docs/spec/cli.md`.
