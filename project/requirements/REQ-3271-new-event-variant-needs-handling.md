---
id: REQ-3271
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-3271

Adding a variant to `Event` MUST NOT compile until every sink that renders a run
handles it.

It replaces a requirement that asked for an unrecognised event to be rendered as
something rather than dropped (from docs/spec/tui.md, high). Nothing can be
unrecognised: `Event` is not `#[non_exhaustive]` and every sink matches it
exhaustively, so a new variant is a compile error in each one (from
docs/spec/tui.md, high). `ShellSink` is outside this, because it renders one
variant on purpose and is not a rendering of a run (from docs/spec/tui.md,
high). The alternative was asking a sink to render what it does not recognise,
which lost because nothing can be unrecognised: the match is exhaustive, so a
new variant is a compile error, which is stronger (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if `Event` becomes
`#[non_exhaustive]` (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-071` in `docs/spec/tui.md`.
