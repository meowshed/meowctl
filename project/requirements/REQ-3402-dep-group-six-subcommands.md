---
id: REQ-3402
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3402

The `dep` group MUST carry `list`, `add`, `remove`, `upgrade`, `sync`, and
`tidy`.

The alternative was to flatten `dep` into top-level commands, which lost because
it adds six more top-level commands to `--help`; revisit if parity is abandoned
(from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-002` in `docs/spec/cli.md`.
