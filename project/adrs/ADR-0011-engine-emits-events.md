---
id: ADR-0011
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1040, REQ-3051, REQ-3280]
supersedes: []
---

# 0011. The engine emits typed events and holds no renderer

## Decision

`meowctl-engine` produces `Event`s and sinks consume them (from
docs/design/0.2.0-decisions.md §2, high). The `Event` enum lives in
`meowctl-common`, below both the producer and the consumers, so a sink never
depends on the engine (from docs/design/0.2.0-rust-rewrite.md §4, high).
REQ-3051 forbids the engine a renderer, REQ-1040 makes the enum cover everything
a sink renders, and REQ-3280 makes every sink testable from a recorded stream
(from docs/spec/engine.md, docs/spec/common.md and docs/spec/tui.md, high).

Once accepted, a test that reads only the event stream sees the whole run (from
https://github.com/meowshed/meowctl/pull/74, high).

## Why

Defect #11 is the concrete cost of the alternative: `buildRunner` takes a
`tui.Writer`, and the consequence is `SuspendOutput`, a renderer concern
threaded into the Starlark layer as a callback so a subprocess could borrow the
terminal (from docs/design/0.2.0-decisions.md §2, high). The event stream also
buys `JsonSink`, which let the compat corpus diff two runs structurally and now
carries `--format json` on every command (from docs/design/0.2.0-decisions.md
§2, high). `v0.1.0` chose the writer on purpose: its ADR-006 took the
`tui.Writer` interface as the lifecycle seam over callback fields or an event
channel (from https://github.com/meowshed/meowctl/issues/17, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Pass a writer into the runner, as `v0.1.0` does | No vocabulary to design up front; a fact is shown where it's produced | It led to `SuspendOutput` in the Starlark layer (defect #11) (from docs/design/0.2.0-decisions.md §2, high) |
| `v0.1.0` ADR-006: a `tui.Writer` passed to the runner, chosen over callback fields or an event channel | One interface as the lifecycle seam | The same defect #11; ADR-006 lived outside this repository and is known only from issue text, so `supersedes` can't link it (from https://github.com/meowshed/meowctl/issues/17, high) |

## What it costs

An indirection between producing a fact and showing it, and a vocabulary that
has to be designed up front; a sink that wants a detail the engine doesn't emit
can't reach for it, and the event has to be added (from
docs/design/0.2.0-decisions.md §2, high).

## What would reverse it

- Nothing short of the event vocabulary proving unable to express something a
  user needs to see, which would be an amendment rather than a reversal (from
  docs/design/0.2.0-decisions.md §2, high).

## Consequences

- `meowctl-tui` depends on `meowctl-common` and on nothing else in the workspace
  (from CLAUDE.md, high).
- M7 could run independently of M4 to M6, because the sinks never depend on the
  engine (from docs/design/0.2.0-rust-rewrite.md §7, high).
- Everything a run says, including a hook's `ctx.log`, subprocess output and a
  failed undo, goes out as an event (from
  https://github.com/meowshed/meowctl/pull/74, high).

## How I will know it was realised

1. `crates/meowctl-engine/tests/running.rs` asserts the event stream carries the
   whole run and holds REQ-3051 (from
   https://github.com/meowshed/meowctl/pull/74 and
   crates/meowctl-engine/tests/running.rs, high).
2. `crates/meowctl-tui/Cargo.toml` lists `meowctl-common` as its only workspace
   dependency (from crates/meowctl-tui/Cargo.toml, high).
3. `crates/meowctl-tui/tests/rendering.rs` renders fixture streams with no
   engine and holds REQ-3280 (from crates/meowctl-tui/tests/rendering.rs, high).

## What this does not settle

- Which sinks exist and when each is chosen; ADR-0013 records that.
- How terminal ownership moves; ADR-0012 records that.
