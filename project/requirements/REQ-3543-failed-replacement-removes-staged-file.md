---
id: REQ-3543
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3543

A replacement of the running binary that fails at any step MUST remove the
staged file.

A staged binary left beside the real one takes over 10 MB and is never cleaned
up, because each run stages under a new process identifier. It is a separate
record from [REQ-3474] so each holds one obligation.

Added during onboarding from the failure-path research; the shipped
`put_in_place` removes the staged file when the rename fails and leaves it when
making it executable fails (from crates/meowctl-cli/src/update.rs:103-117,
high).
