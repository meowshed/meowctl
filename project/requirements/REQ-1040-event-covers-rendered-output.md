---
id: REQ-1040
artifact: requirement
topic: common
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-1040

The `Event` enum MUST cover everything a command produces that a sink renders.
At minimum: `PlanComputed`, `PhaseStarted`, `PhaseFinished`, `ComponentStarted`,
`ComponentSkipped`, `ComponentFinished`, `OpApplied`, `ProcessStarted`,
`ProcessOutput`, `ProcessFinished`, `TerminalRequested`, `TerminalReleased`,
`Diagnostic`, and `Message { severity, text }`.

The alternative was a smaller event set, extended when needed, which lost
because a sink can't show what isn't emitted, and the engine is the awkward
place to add one later; revisit it if an event proves unused after the sinks
exist (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-COMMON-040` in `docs/spec/common.md`.
