---
id: ADR-0044
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3414]
supersedes: []
---

# 0044. Handle signals in the binary, on a thread woken by signal-hook

## Decision

The binary installs the handler for interruption and suspension, and the work
happens on a thread woken by `signal-hook`'s iterator rather than in a signal
handler; the thread writes the escape and re-raises with the default disposition
(from https://github.com/meowshed/meowctl/pull/76, high). REQ-3414 places the
handler in the binary (from docs/spec/cli.md, high).

Once accepted, the cursor is restored and the shell sees the process die the way
it expects (from https://github.com/meowshed/meowctl/pull/76, high).

## Why

A handler runs between any two instructions and may call only async-signal-safe
functions, and writing an escape through a locked `Stdout` is neither (from
https://github.com/meowshed/meowctl/pull/76, high). A signal arrives at the
process, and a library taking a process-wide resource would surprise every other
caller (from https://github.com/meowshed/meowctl/pull/71, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Do the work inside the signal handler | No thread | Writing through a locked `Stdout` isn't async-signal-safe (from https://github.com/meowshed/meowctl/pull/76, high) |
| Let the sink install its own handler | The sink owns its own clean-up | A library taking a process-wide resource surprises every other caller (from https://github.com/meowshed/meowctl/pull/71, high) |

## What it costs

A thread that lives for the process (from crates/meowctl-cli/src/signals.rs,
medium).

## What would reverse it

- The source names no reversal condition (from
  https://github.com/meowshed/meowctl/pull/76, high).

## Consequences

- The first `SIGINT` later became a flag the runner reads between components
  rather than a stop (from https://github.com/meowshed/meowctl/pull/86, high).

## How I will know it was realised

1. `crates/meowctl-cli/src/signals.rs` uses `signal_hook::iterator::Signals` and
   `std::thread::spawn`, and `tests/interrupt.rs` sends the binary a real
   `SIGINT` and holds REQ-3414 (from those files and
   https://github.com/meowshed/meowctl/pull/86, high).

## What this does not settle

- How an interrupt stops the run; the requirements from `docs/spec/cli.md` and
  `docs/spec/engine.md` on interruption carry that (from
  https://github.com/meowshed/meowctl/pull/86, high).
