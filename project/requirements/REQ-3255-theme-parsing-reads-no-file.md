---
id: REQ-3255
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3255

Theme parsing MUST NOT read a file.

`meowctl-tui` depends on `meowctl-common` and on nothing else in the workspace,
so it takes the text and the caller brings it; see REQ-3410 (from
docs/spec/tui.md, high). The alternative was giving `meowctl-tui` a
`FileSystem`, which lost because it would be its only workspace dependency
beyond `meowctl-common`, for one read (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-055` in `docs/spec/tui.md`.
