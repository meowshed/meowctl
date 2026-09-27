---
id: REQ-2251
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2251

A builtin called with wrong or missing arguments MUST name the builtin, the
argument, and what was expected.

'Invalid arguments' sends the author reading the source of a binary they don't
have (from crates/meowctl-starlark/tests/evaluation.rs:586-588, high).
`starlark-rust`'s own message already names all three, and this requirement
keeps it so (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-STAR-051` in `docs/spec/starlark.md`.
