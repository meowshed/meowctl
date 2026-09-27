---
id: REQ-3241
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3241

Motion MUST be disabled for a pipe, for `TERM` unset or `dumb`, and when `CI` is
set, even when a pty is present.

`DetectCaps` is deliberately conservative here, and cursor movement written into
a CI transcript is unreadable (from docs/spec/tui.md, high). The alternative was
trusting the pty, which lost because cursor movement in a CI transcript is
unreadable (from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it
if a CI provider renders motion properly and users ask (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-041` in `docs/spec/tui.md`.
