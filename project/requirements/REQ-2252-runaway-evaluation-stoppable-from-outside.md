---
id: REQ-2252
artifact: requirement
topic: starlark
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2252

A configuration that loops forever MUST be stoppable from outside the
evaluation.

Neither binary bounds an evaluation: `go.starlark.net`'s step limit is off
unless `SetMaxExecutionSteps` is called, which `v0.1.0` never calls, and
`starlark-rust` 0.14 has no step limit (from docs/spec/starlark.md, high). The
only statement hook in `starlark-rust`, `before_stmt`, is `pub(crate)`, its
`#[doc(hidden)]` DAP alias is documented as not public API, and its callback
cannot fail an evaluation (from docs/spec/starlark.md, high). So the process is
what stops a runaway configuration: the first interrupt sets a flag the engine
reads between components, which an evaluation that never returns never reaches,
and the second interrupt kills the process, which is what REQ-3453 is for (from
docs/spec/starlark.md, high). The alternative was to claim a bound neither
binary has, which lost because `go.starlark.net`'s step limit is off unless set,
`v0.1.0` never sets it, and `starlark-rust` has no supported hook at all;
revisit if `starlark-rust` gains one (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

No test cites this yet; the evidence still missing is a test in
`tests/interrupt.rs` that sends two interrupts to an evaluation looping over
`range(1000000000000)` and asserts that the process ends by signal 2 within 10 s
(from crates/meowctl-cli/src/signals.rs:59-65, high).

Migrated from `R-STAR-052` in `docs/spec/starlark.md`, its first obligation.
