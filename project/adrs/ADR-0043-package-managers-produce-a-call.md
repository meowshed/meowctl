---
id: ADR-0043
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2614]
supersedes: []
---

# 0043. Have meowctl-pm produce a Call for the engine to make

## Decision

`meowctl-pm` produces a `Call`: which component's file to evaluate, which
function to call, and with what; the engine makes the call with the `ctx` of the
component that asked for the package (from
https://github.com/meowshed/meowctl/pull/69, high). REQ-2614 is the requirement
it serves (from docs/spec/pm.md, high). `query_pm` goes out through a trait the
engine supplies, so `meowctl-pm` sits beside `meowctl-ctx` rather than
underneath the evaluator (from https://github.com/meowshed/meowctl/pull/69,
high).

Once accepted, no handler is held across evaluations.

## Why

`v0.1.0` keeps live `Callable` values in its registry, which works because its
heap outlives every evaluation; in `starlark-rust` a heap is scoped to one, so a
handler can't be held across evaluations at all (from
https://github.com/meowshed/meowctl/pull/69, high). Holding a `Callable` also
made passing the handler's `ctx` rather than the caller's easy to get wrong
(from https://github.com/meowshed/meowctl/pull/69, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Keep live `Callable` values in the registry, as `v0.1.0` does | Dispatch calls the handler directly | A `starlark-rust` heap is scoped to one evaluation, so the value can't outlive it (from https://github.com/meowshed/meowctl/pull/69, high) |

## What it costs

Three requirements about when a handler is called moved to the runner, which
makes the call (from https://github.com/meowshed/meowctl/pull/69, high).

## What would reverse it

- The source names no reversal condition (from
  https://github.com/meowshed/meowctl/pull/69, high).

## Consequences

- REQ-2603, REQ-2615 and REQ-2621 moved to issue 30 (from
  https://github.com/meowshed/meowctl/pull/69, high).

## How I will know it was realised

1. `crates/meowctl-pm/src/registry.rs` defines `pub struct Call`, and
   `crates/meowctl-pm/tests/registry.rs` holds REQ-2614 (from those files,
   high).

## What this does not settle

- The source recorded no alternative beyond live `Callable` values (from
  https://github.com/meowshed/meowctl/pull/69, high).
