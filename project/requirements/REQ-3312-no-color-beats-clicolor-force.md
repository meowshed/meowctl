---
id: REQ-3312
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3312

`NO_COLOR` MUST still win over `CLICOLOR_FORCE`.

A caller that renders meowctl's output itself -- a CI log viewer, a pager
invoked deliberately -- has no way to say so otherwise, because from here it
looks exactly like a pipe to a file (from docs/spec/tui.md, high). The
precedence is the convention's: `NO_COLOR` is a user saying they do not want
colour anywhere, and that outranks a caller saying this particular pipe can take
it (from docs/spec/tui.md, high). The alternative was never colouring a pipe,
which lost because a caller rendering the output itself looks exactly like a
pipe to a file from here (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-046` in `docs/spec/tui.md`, its second obligation.
