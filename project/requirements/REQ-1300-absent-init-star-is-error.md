---
id: REQ-1300
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1300

An absent `init.star` MUST be an error.

The alternative was treating every absent file as an error, which lost because a
first run has no lock and no state, and erroring makes `init` the only working
command, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-003` in `docs/spec/config.md`, its second obligation.
