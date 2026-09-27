---
id: REQ-1262
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1262

A write that fails MUST leave the file it was replacing intact; see [REQ-1403]
and [REQ-1431].

A write that fails partway would leave a lock or state file the next run can't
parse and no previous file to fall back on (from
docs/design/0.2.0-requirement-tradeoffs.md:62, high). The atomic rename
[REQ-1403] requires is what provides this, so no separate mechanism is needed
(from docs/design/0.2.0-execution-plan.md:147-150 and
https://github.com/meowshed/meowctl/pull/67, high).

Migrated from `R-CONFIG-062` in `docs/spec/config.md`.
