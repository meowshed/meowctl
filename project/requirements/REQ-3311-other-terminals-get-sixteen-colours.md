---
id: REQ-3311
artifact: requirement
topic: tui
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3311

Anything other than a `COLORTERM` naming `truecolor` or `24bit` or a `TERM`
containing `256` MUST be the 16-colour depth.

These two variables are how a terminal reports what it takes, which is the
"reports" in REQ-3308 (from docs/spec/tui.md, high). A reader whose theme looks
wrong needs to know which variable to check, and a `COLORTERM` this misreads is
indistinguishable from a palette that is simply ugly (from docs/spec/tui.md,
high). The alternative was leaving "the depth the terminal reports" abstract,
which lost because a reader whose colours look wrong needs to know which
variable to check (from docs/design/0.2.0-requirement-tradeoffs.md, high). The
trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-045` in `docs/spec/tui.md`, its second obligation.
