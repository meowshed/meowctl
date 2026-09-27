---
id: REQ-3431
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3431

An error carrying a source span MUST be rendered as a diagnostic showing the
file, the line, and the offending source text; see [REQ-2240].

The alternative was to print the message, which lost because the M0 spike showed
the span is available and the caret rendering costs nothing, and the trade-off
record names no condition that reverses it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-CLI-031` in `docs/spec/cli.md`.
