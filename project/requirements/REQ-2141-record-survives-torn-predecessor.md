---
id: REQ-2141
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2141

A journal record MUST stay readable on its own when the record before it was
torn.

Replay skips a line it can't parse [REQ-2030], so a record joined onto a torn
one loses its inverse along with the torn record.

Added during onboarding from the failure-path research; the shipped `append`
writes after whatever the file ends with, so a partial `writeln` leaves a line
with no newline and the next record joins it (from
crates/meowctl-ops/src/journal.rs:155-162, medium: inferred from the code, not
probed).
