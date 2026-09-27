---
id: REQ-1250
artifact: requirement
topic: config
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1250

Adding or removing a `component()` declaration in `init.star` or `local.star`
MUST preserve every comment, every blank line, and the formatting of every
statement it does not touch.

The file belongs to the user, and a round trip through a parsed model and a
printer would lose the comments they wrote and the grouping they chose, so the
edit is made to the text at the position the parse reports (from
docs/spec/config.md, high). The alternative was rewriting the file from the
parsed model, which lost because every comment a user wrote disappears on
`meowctl add`, and the record expects nothing to reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CONFIG-050` in `docs/spec/config.md`.
