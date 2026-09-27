---
id: REQ-3433
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3433

An error MUST go to stderr.

The alternative was to write errors to stdout, which lost because
`meowctl dep list | grep` would ingest the error text, and the trade-off record
names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-033` in `docs/spec/cli.md`, its first obligation.
