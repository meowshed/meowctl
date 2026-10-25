---
id: TSK-0042
artifact: task
status: approved
revised: 2026-10-25
realises: ADR-0051
closes: [REQ-2915]
---

# Keep `add_path` on the read-only `ctx`

A hook in a read-only phase can call `ctx.add_path`, and still finds no other
mutating method, so the stdlib `verify` hooks that activate mise's shims run.

## Acceptance criteria

1. Given the read-only surface, when a hook calls `ctx.add_path("/opt/bin")`,
   then the call succeeds and a `PathPrepended` event for `/opt/bin` is
   emitted. Closed by: a test in `crates/meowctl-ctx/tests/surface.rs` naming
   REQ-2915.
2. Given the read-only surface, when a hook reads any other mutating method,
   then it gets attribute-not-found. Closed by:
   `a_read_only_surface_has_no_mutating_method`, with `add_path` taken off its
   list.

## What to do

Move `add_path` in `crates/meowctl-ctx/src/value.rs` from the mutating methods
to the methods the full and read-only surfaces share, and cite REQ-2915 where
the code and tests cite REQ-2830.

## Depends on

None.

## Left alone

The shell surface keeps its eight attributes from REQ-2831, which don't
include `add_path`.

## Evidence

Not yet.
