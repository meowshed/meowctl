---
id: REQ-3256
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3256

The theme MUST be read once, before the sink is built.

The alternative was warning about the absent file too, which lost because almost
nobody has one, so it would warn on every command about a file the user never
wrote (from docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off
table names no condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-056` in `docs/spec/tui.md`, its first obligation.
