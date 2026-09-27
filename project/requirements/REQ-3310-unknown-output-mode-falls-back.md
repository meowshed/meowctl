---
id: REQ-3310
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3310

An unrecognised `MEOWCTL_OUTPUT` value MUST fall back to detection rather than
erroring.

`ParseMode` does this in `v0.1.0`, and failing mid-apply over a typo in an
environment variable is the behaviour to avoid (from docs/spec/tui.md, high).
The alternative was failing on an unrecognised value, which lost because a typo
in an environment variable would fail an apply mid-run (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-044` in `docs/spec/tui.md`, its second obligation.
