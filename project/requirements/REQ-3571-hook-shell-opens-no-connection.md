---
id: REQ-3571
artifact: requirement
topic: cli
class: non-functional
status: approved
revised: 2026-09-27
elaborates:
verification: static
---

# REQ-3571

`meowctl hook shell` MUST NOT open a network connection or start an
asynchronous runtime.

`meowctl hook shell` runs on every shell spawn, so whatever it starts is paid
for thousands of times a day, and the plan lists startup cost as a goal for
that reason (from docs/design/0.2.0-rust-rewrite.md:27-28 and
docs/design/0.2.0-decisions.md:132-135, high).

Added during onboarding from the research on decision 8; ADR-0008 trades
against this and no requirement stated it (from docs/design/0.2.0-decisions.md
§2, high). The shipped binary has no
asynchronous runtime in its dependency tree (from Cargo.lock, high).
