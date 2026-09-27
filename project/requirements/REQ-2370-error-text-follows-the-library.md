---
id: REQ-2370
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2370

A Starlark runtime error MUST report the same mistake `v0.1.0` reports, in the
wording `starlark-rust` produces.

The wording isn't an interface a component depends on, and a table translating
it would track a dependency's internal strings and break the first time the
library adds a message; the mistake reported is part of parity (from
docs/spec/starlark.md:200-203 and docs/design/0.2.0-decisions.md §3.11, high).

Added during onboarding from the research on decision 8; the exemption was
stated in the specification with no identifier, and ADR-0029 decides it (from
docs/design/0.2.0-decisions.md:15-16, high).
