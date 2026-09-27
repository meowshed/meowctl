---
id: REQ-3341
artifact: requirement
topic: tui
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3341

A prompt MUST NOT wait for input unless its question was written to a terminal.

The user must see a question before the process waits for its answer, or the run
hangs on a question nobody saw.

Added during onboarding from the failure-path research; the shipped prompt
counts a session as interactive when stdin is a terminal, whatever stderr is
(from crates/meowctl-cli/src/run.rs:257-261 and
crates/meowctl-tui/src/interaction.rs:81-137, high).
