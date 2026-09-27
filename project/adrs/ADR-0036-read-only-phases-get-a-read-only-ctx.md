---
id: ADR-0036
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2830]
supersedes: []
---

# 0036. Give a read-only phase a ctx with no mutating methods

## Decision

In `install_check`, `upgrade_check`, `uninstall_check` and `verify`, `ctx`
exposes no mutating method, and a hook that tries to write gets
attribute-not-found (from docs/design/0.2.0-decisions.md §3.20 and
docs/spec/ctx.md, high). REQ-2830 states it (from docs/design/0.2.0-decisions.md
§3.20, high).

Once accepted, a check can't have a side effect. This is a change from `v0.1.0`,
which has the machinery and wires none of it up (from
docs/design/0.2.0-decisions.md §3.20, high).

## Why

A check that writes is a check with a side effect, and the phase names say what
they're for (from docs/design/0.2.0-decisions.md §3.20, high). The risk was
measured: no `verify`, `install_check`, `upgrade_check` or `uninstall_check`
hook in `meowctl-stdlib` or `dotmeow` calls a mutating method (from
docs/design/0.2.0-decisions.md §3.20, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Leave every phase with the full `ctx`, as `v0.1.0` does | No component that writes during a check starts failing | A check could have side effects the phase name denies (from docs/design/0.2.0-decisions.md §3.20, high) |

## What it costs

A component that writes during a check starts failing (from
docs/design/0.2.0-decisions.md §3.20, high).

## What would reverse it

- A component turns up with a real need to write during a check, which would
  mean the phase split is wrong rather than the restriction (from
  docs/design/0.2.0-decisions.md §3.20, high).

## Consequences

- The restricted surface is a distinct Starlark type sharing one core, not a
  check inside each method (from https://github.com/meowshed/meowctl/pull/70,
  high).

## How I will know it was realised

1. `crates/meowctl-ctx/tests/surface.rs` holds REQ-2830, and
   `crates/meowctl-ctx/src/value.rs` defines `ReadOnlyCtx` (from those files,
   high).

## What this does not settle

- The source recorded no alternative beyond the full `ctx` (from
  docs/design/0.2.0-decisions.md §3.20, high).
