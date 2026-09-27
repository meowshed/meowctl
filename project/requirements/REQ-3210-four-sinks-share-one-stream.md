---
id: REQ-3210
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3210

Four sinks MUST consume the same stream: `LiveSink` for a capable terminal,
`PlainSink` for everything else, `JsonSink` for `--format json`, and `ShellSink`
for `meowctl hook`.

An earlier draft named three, and `ShellSink` was missed because `hook` was the
last command to be written (from docs/spec/tui.md, high). `ShellSink` is not a
rendering of a run at all, because its output is evaluated by a shell (from
docs/spec/tui.md, high). The alternative was one sink with mode flags, which
lost because the sinks differ in structure, not in decoration, and flags would
fork every method (from docs/design/0.2.0-requirement-tradeoffs.md, high).
Revisit it if a new output mode differs from an existing sink only in decoration
(from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off row said
"a fourth output mode is mostly like an existing one" and predates `ShellSink`,
which is the fourth and is like none of the others, because it drops every
event except `ShellLine`; onboarding restated the condition (from
crates/meowctl-tui/src/sink.rs:265-298 and
https://github.com/meowshed/meowctl/pull/82, high).

Migrated from `R-TUI-010` in `docs/spec/tui.md`.
