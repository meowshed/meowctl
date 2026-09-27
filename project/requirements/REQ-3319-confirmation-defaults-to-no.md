---
id: REQ-3319
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3319

A confirmation MUST treat a bare Enter and end-of-input as no.

`Printer.Confirm` in `v0.1.0` is this behaviour, and it exists because
`fmt.Scanln` failed the command on an empty line (from docs/spec/tui.md, high).
The alternative was reading with a simpler call, which lost because `fmt.Scanln`
failed the command on a bare Enter, which is the advertised default (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-061` in `docs/spec/tui.md`, its second obligation.
