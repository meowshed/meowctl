---
id: REQ-3281
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3281

Snapshots MUST cover the capability matrix: motion, colour depth, glyph tier,
and width.

`v0.1.0`'s tests check a few cases by hand, which is why the matrix is stated as
an obligation (from docs/spec/tui.md, high). The alternative was spot-checking a
few cells, which lost because that is `v0.1.0`, and the untested cells are where
the degradation bugs are (from docs/design/0.2.0-requirement-tradeoffs.md,
high). Revisit it if the matrix grows enough that the snapshots dominate the
suite (from docs/design/0.2.0-requirement-tradeoffs.md, high).
`docs/design/0.2.0-decisions.md` argues the choice at length (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-TUI-081` in `docs/spec/tui.md`.
