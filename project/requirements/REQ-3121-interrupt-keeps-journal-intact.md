---
id: REQ-3121
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3121

A run interrupted by a signal MUST leave the journal intact for the next run to
find; see REQ-3042.

The engine learns of the interrupt through a flag the caller owns and the runner
reads, and nothing below `meowctl-cli` installs a signal handler (from
docs/spec/engine.md, high). The flag interrupts the gap between components and
not a component, because a hook halfway through `brew install` is not something
the engine can stop, and a terminal's interrupt reaches the whole process group
anyway (from docs/spec/engine.md, high). The alternative was stopping
immediately, mid-component, which lost because a component interrupted mid-hook
leaves a state the journal describes but the sentinel does not (from
docs/design/0.2.0-requirement-tradeoffs.md, high). The trade-off table names no
condition that would reverse it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-062` in `docs/spec/engine.md`, its second obligation.
