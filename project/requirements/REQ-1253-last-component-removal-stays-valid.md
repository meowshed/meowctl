---
id: REQ-1253
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1253

Removing the last `component()` from a file MUST leave a valid file rather than
an empty one, because the next `add` has to have somewhere to write.

The next `add` has to have somewhere to write (from docs/spec/config.md, high).
The alternative was allowing an empty file, which lost because `meowctl add`
after `meowctl remove` of the last component would have nothing to parse, and
the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-053` in `docs/spec/config.md`.
