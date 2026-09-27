---
id: REQ-1203
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1203

A file that does not exist MUST parse as its empty value where `v0.1.0` treats
it that way: an absent lock file is a clean slate, an absent `state.toml` is a
first run, an absent `local.star` is no local components.

The alternative was treating every absent file as an error, which lost because a
first run has no lock and no state, and erroring makes `init` the only working
command, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-003` in `docs/spec/config.md`, its first obligation.
