---
id: REQ-3223
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3223

The cursor MUST be restored on exit, on interruption, and on suspension.

A command that exits with a hidden cursor leaves the user's shell broken (from
docs/spec/tui.md, high). A signal arrives at the process rather than at the
sink, so `meowctl-cli` installs the handler and calls the sink; see REQ-3414
(from docs/spec/tui.md, high). The alternative was restoring the cursor on the
normal exit path only, which lost because a Ctrl-C leaves the shell with no
cursor (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-023` in `docs/spec/tui.md`, its first obligation.
