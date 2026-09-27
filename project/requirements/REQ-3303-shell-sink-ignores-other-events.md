---
id: REQ-3303
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3303

`ShellSink` MUST write nothing for any event other than `ShellLine`, because
whatever it wrote the shell would evaluate; see REQ-3461 and REQ-1044.

`ShellSink` is the exception to rendering every event and is not a rendering of
a run, because whatever it wrote the shell would evaluate; see REQ-3461 and
REQ-1044 (from docs/spec/tui.md, high). The alternative was letting a sink drop
what it cannot show, which lost because a command's output would depend on where
it runs (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-012` in `docs/spec/tui.md`, its third obligation.
