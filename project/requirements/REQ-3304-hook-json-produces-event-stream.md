---
id: REQ-3304
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3304

`--format json` on `hook` MUST still produce the event stream, because a program
reading events is not a shell evaluating them.

The alternative was letting `--format json` on `hook` emit shell code, which
lost because a program reading events is not a shell evaluating them, and one of
the two would get the wrong thing (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-013` in `docs/spec/tui.md`, its second obligation.
