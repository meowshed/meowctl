---
id: REQ-3001
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-3001

Discovery, resolution, planning, and execution MUST be distinct types, so a
stage cannot be entered before the one it depends on has produced its value.

`v0.1.0` expresses the same ordering in comments, which is how "pass one
registers PM handlers" became something a caller can forget (from
docs/spec/engine.md, high). No test case holds it: every engine test walks the
stages in order because each takes what the one before returned, and the
alternative does not compile (from crates/meowctl-engine/tests/running.rs,
high). The alternative was one struct that gains fields as it goes, which lost
because that is `v0.1.0`, where the pass ordering is a comment a caller can
ignore (from docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if
the stages collapse into one genuinely atomic step (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-001` in `docs/spec/engine.md`.
