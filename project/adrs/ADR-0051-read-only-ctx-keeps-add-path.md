---
id: ADR-0051
artifact: adr
status: approved
revised: 2026-10-25
addresses: [REQ-2915]
supersedes: [ADR-0036]
---

# 0051. Give a read-only phase a ctx with no mutating methods, keeping `add_path`

## Decision

In `install_check`, `upgrade_check`, `uninstall_check` and `verify`, `ctx`
exposes no method that changes the machine, and a hook that tries one gets
attribute-not-found, as ADR-0036 decided. `add_path` stays on that surface,
because it changes only the `PATH` the rest of this run hands to `ctx.run`,
and nothing outside the meowctl process sees it. REQ-2915 states it.

Once accepted, the 100 meowctl-stdlib `verify` hooks that call `add_path`
through `_activate_shims` and similar helpers run again, and a check still
can't write a file, run a download or change a default.

## Why

ADR-0036 rested on a measurement that no check hook in meowctl-stdlib or
dotmeow calls a mutating method, and named the reversal: a component with a
real need to write during a check. The measurement counted `add_path` as
mutating and missed that 100 stdlib `verify` hooks call it, starting with
`@stdlib//components/mise:36`, so `meowctl verify` stops at the first of them
(from BUG-0041's reproduction run, high). Those hooks need it for a real
reason: a tool mise installed is on the `PATH` only through its shims, and
`verify` runs that tool. `add_path` only emits a `PathPrepended` event that
the executor folds into later commands, so a check that calls it has no side
effect the phase name denies (from `crates/meowctl-ctx/src/value.rs`, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Keep ADR-0036 and change the 100 stdlib hooks | The read-only surface stays the set ADR-0036 listed | Each hook would have to find its tool another way, and `v0.1.0` runs those hooks as they are, which the parity principle keeps |
| Give a read-only phase the full `ctx`, as `v0.1.0` does | No component that writes during a check fails | A check could write files and run downloads, the side effect ADR-0036 removed |

## What it costs

The read-only surface has one method that isn't a pure read, so "read-only"
means "changes nothing outside this run" and not "calls nothing that sets
state".

## What would reverse it

- `add_path` starts changing something outside the meowctl process, such as a
  shell profile, which would make it mutating in ADR-0036's sense.

## Consequences

- REQ-2915 supersedes REQ-2830, and SPC-2800 states it in place of REQ-2830.

## How I will know it was realised

1. `crates/meowctl-ctx/tests/surface.rs` calls `add_path` on the read-only
   surface and holds REQ-2915, and the same test still finds no other mutating
   method there.
