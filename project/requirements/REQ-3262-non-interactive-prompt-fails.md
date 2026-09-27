---
id: REQ-3262
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3262

In a non-interactive session, a prompt MUST fail with a message naming what it
wanted, rather than blocking.

A CI run that hangs on a prompt until it times out is the failure to prevent
(from docs/spec/tui.md, high). The alternative was blocking while waiting for
input, which lost because a CI run hangs until it times out (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a
non-interactive default answer is wanted more than a failure (from
docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-062` in `docs/spec/tui.md`.
