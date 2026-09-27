---
id: REQ-3453
artifact: requirement
topic: cli
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3453

A second interrupt MUST stop the process at once, with the default disposition.

A user who has asked twice is not asking for a tidier stop: the first interrupt
asks the run to finish what it is doing and stop, and if that takes longer than
they will wait, the second has to work, because a process that swallows its own
interrupt is one nobody can stop (from docs/spec/cli.md, high). The process dies
from the second interrupt, which is where the shell's 130 comes from, as it does
under `v0.1.0` (from docs/spec/cli.md, high). The alternative was to let the
first interrupt be the only one, which lost because a run that will not stop
when asked twice is one nobody can stop, and the trade-off record names no
condition that reverses it (from docs/design/0.2.0-requirement-tradeoffs.md,
high).

No test sends a second interrupt yet, since `tests/interrupt.rs:42-86` sends
one; the evidence still missing is the two-interrupt test [REQ-2252] also needs
(from tests/interrupt.rs:42-86, high).

Migrated from `R-CLI-053` in `docs/spec/cli.md`.
