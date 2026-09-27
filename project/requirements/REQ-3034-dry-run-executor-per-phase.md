---
id: REQ-3034
artifact: requirement
topic: engine
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-3034

During a dry run the engine MUST give each phase an executor built for that
phase, because whether a command runs depends on whether the phase is read-only;
see REQ-1611.

Choosing an implementation per phase differs from a method branching on a flag:
no code below `meowctl-cli` asks whether this is a dry run while it is doing
something, and REQ-2814 still holds (from docs/spec/engine.md, high). The engine
is the only place that knows both the flag and the phase, because the command
line knows the flag and not the phases, and the executor knows the phase it was
built for and not the flag (from docs/spec/engine.md, high). The alternative was
building one executor for the whole run, which lost because whether a command
runs depends on the phase, and the command line does not know the phases (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if the read-only
distinction disappears (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-ENGINE-034` in `docs/spec/engine.md`.
