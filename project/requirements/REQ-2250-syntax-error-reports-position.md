---
id: REQ-2250
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2250

A file that does not parse MUST report the syntax error with its position.

A syntax error without a position sends the author searching the file for it
(inferred during onboarding, low), and `starlark-rust` reports a span and a
caret where `v0.1.0` names only the file (from ADR-0029, high).

Migrated from `R-STAR-050` in `docs/spec/starlark.md`, its first obligation.
