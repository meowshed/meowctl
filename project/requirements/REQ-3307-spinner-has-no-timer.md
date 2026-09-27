---
id: REQ-3307
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3307

The spinner MUST NOT be driven by a timer.

`v0.1.0` animates the spinner from a goroutine, which means a renderer owns a
thread, a mutex and a shutdown path, and a test of the output has to wait for
wall-clock time to pass (from docs/spec/tui.md, high). The cost is real and
bounded: a component whose subprocess runs silently for a minute shows a still
spinner for that minute, and since captured output and every other event redraw
it, the case is a command that produces nothing at all (from docs/spec/tui.md,
high). The alternative was animating from a timer thread, as `v0.1.0` does,
which lost because a renderer that owns a thread owns a mutex and a shutdown
path, and its output cannot be tested without waiting (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if a silent
long-running command is common enough to notice (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-026` in `docs/spec/tui.md`, its second obligation.
