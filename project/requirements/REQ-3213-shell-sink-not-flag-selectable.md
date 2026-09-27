---
id: REQ-3213
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3213

`ShellSink` MUST NOT be selectable by a flag.

`hook` chooses `ShellSink` (from docs/spec/tui.md, high). The alternative was
letting `--format json` on `hook` emit shell code, which lost because a program
reading events is not a shell evaluating them, and one of the two would get the
wrong thing (from docs/design/0.2.0-requirement-tradeoffs.md, high). The
trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-013` in `docs/spec/tui.md`, its first obligation.
