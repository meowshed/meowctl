---
id: TSK-0042
artifact: task
status: done
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

Criterion 1 is closed by `a_read_only_surface_keeps_add_path`, which failed
before the change and passes after it, and criterion 2 by
`a_read_only_surface_has_no_mutating_method`, which still lists the twelve other
mutating methods. `mise run fmt-check`, `mise run check`, `mise run test` (607
passed, 3 skipped after rebasing onto #168) and `mise run build` exit 0. A
release build of this branch run as `meowctl verify` against a real
configuration passed `@stdlib//components/mise`, where `main` stops, and next
stopped at `@stdlib//components/mprocs`, whose tool upstream renamed to `dekit`.
