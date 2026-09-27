---
id: ADR-0012
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1621, REQ-1622, REQ-3222]
supersedes: []
---

# 0012. Negotiate terminal ownership through events, and delete SuspendOutput

## Decision

An interactive command emits `TerminalRequested`, the sink stands down and
erases its region, and `TerminalReleased` brings it back; the negotiation is
between the executor and the sink, so `SuspendOutput` disappears from `ctx`
(from docs/design/0.2.0-rust-rewrite.md §4, high). The callback must not exist
anywhere in the tree (from docs/design/0.2.0-decisions.md §3.6, high). REQ-1621,
REQ-1622 and REQ-3222 state it (from docs/spec/exec.md and docs/spec/tui.md,
high).

Once accepted, nothing in `meowctl-exec` knows a renderer exists, and the
release is emitted even when the command fails (from
https://github.com/meowshed/meowctl/pull/58, high).

## Why

The callback only ever existed to paper over the engine holding a renderer (from
docs/design/0.2.0-rust-rewrite.md §4, high). It requires the Starlark layer to
hold a function whose purpose is to reach into a renderer, which is the concrete
form of defect #11 and the reason `ctx.Capabilities` knows what a terminal is
(from docs/design/0.2.0-decisions.md §3.6, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Port the `SuspendOutput` callback | It works and is small | The Starlark layer would hold a function that reaches into a renderer (from docs/design/0.2.0-decisions.md §3.6, high) |

## What it costs

Two event variants and a sink that has to honour them: the live sink writes
nothing between `TerminalRequested` and `TerminalReleased` (from
https://github.com/meowshed/meowctl/pull/71, high).

## What would reverse it

- The source names no reversal condition for this requirement; the trade-off
  table marks REQ-1622 as decided (from docs/design/0.2.0-decisions.md §3.6 and
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- M7's done-when includes `SuspendOutput` existing nowhere in the tree (from
  docs/design/0.2.0-rust-rewrite.md §6, high).
- `TerminalReleased` is emitted even when the process fails or the run is
  interrupted, so the user keeps a cursor (from docs/spec/exec.md, high).

## How I will know it was realised

1. `crates/meowctl-exec/tests/execution.rs` holds REQ-1621 and REQ-1622, and
   `crates/meowctl-tui/tests/live.rs` holds REQ-3222 (from those files, high).
2. `rg SuspendOutput crates` finds the name only in three module doc comments
   that explain why it's gone (from crates/meowctl-engine/src/runner.rs,
   crates/meowctl-engine/src/lib.rs and crates/meowctl-exec/src/lib.rs, high).

## What this does not settle

- The source recorded no alternative beyond porting the callback (from
  docs/design/0.2.0-decisions.md §3.6, high).
