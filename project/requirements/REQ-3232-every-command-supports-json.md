---
id: REQ-3232
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3232

Every command MUST support `--format json`.

In `v0.1.0` only `doctor` has a JSON form, hand-written; here it falls out of
the sink (from docs/spec/tui.md, high). The alternative was porting
`doctor --json` as a special case, which lost because every other command stays
unscriptable and the corpus has nothing to diff (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-032` in `docs/spec/tui.md`.
