---
id: REQ-3270
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3270

A write to the output destination that fails MUST NOT panic.

A closed pipe is normal when output is piped into `head` (from docs/spec/tui.md,
high). The alternative was letting the write error propagate, which lost because
`meowctl status | head` would report a broken pipe as a failure (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-070` in `docs/spec/tui.md`, its first obligation.
