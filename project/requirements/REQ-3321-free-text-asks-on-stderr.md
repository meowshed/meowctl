---
id: REQ-3321
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3321

The `Interaction` trait MUST send a free-text question to stderr for the same
reason a confirmation does.

The trait has two methods rather than a confirmation built on the free-text one,
because a confirmation has a default and a fixed vocabulary, and the sink that
renders it and the session that cannot answer it both need to know which was
asked (from docs/spec/tui.md, high). The alternative was building the
confirmation on a free-text prompt, which lost because a confirmation has a
default and a fixed vocabulary, and a non-interactive session has to know which
of the two it is refusing (from docs/design/0.2.0-requirement-tradeoffs.md,
high). The trade-off table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-063` in `docs/spec/tui.md`, its second obligation.
