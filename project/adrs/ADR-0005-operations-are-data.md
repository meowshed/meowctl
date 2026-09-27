---
id: ADR-0005
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2001, REQ-2017, REQ-2813, REQ-2002, REQ-2102]
supersedes: []
---

# 0005. Make every reversible effect a variant of Op with an inverse

## Decision

`meowctl-ops::Op` is an enum, not a set of methods: `Op::inverse()` produces the
undo record, `Op::apply(&dyn FileSystem)` performs it, and the journal is a log
of `Op`s (from docs/design/0.2.0-rust-rewrite.md §3, high). REQ-2001 names the
variants a journal carries, REQ-2813 sends every mutating `ctx` method through
an `Op`, and REQ-2017 gives `defaults_write` and `plist_set` an inverse by
recording the prior value (from docs/spec/ops.md, docs/spec/ctx.md and
docs/design/0.2.0-decisions.md §2, high).

Once accepted, adding an effect without an undo doesn't compile. An effect with
no readable prior state still has nowhere honest to live (from
docs/design/0.2.0-decisions.md §2, high).

## Why

The `v0.1.0` arrangement lets a new `ctx` method ship with no rollback, and
nothing notices until a failed run leaves a machine half configured; the enum
makes the compiler ask (from docs/design/0.2.0-decisions.md §2, high). `v0.1.0`
declares the `defaults_write` and `plist_set` kinds, returns "inverse not
implemented" for them, and never journals either (from
https://github.com/meowshed/meowctl/pull/59, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Methods that perform an effect and separately append to a journal, as `v0.1.0` does | One new method per effect | A new method can ship with no rollback and nothing notices (from docs/design/0.2.0-decisions.md §2, high) |

## What it costs

The enum is closed: adding an effect touches the enum, its apply, its inverse
and the journal's serialization, where `v0.1.0` needed one method (from
docs/design/0.2.0-decisions.md §2, high). An effect that can't be undone has
nowhere to live without a variant that lies about having an inverse (from
docs/design/0.2.0-decisions.md §2, high).

## What would reverse it

- An effect must be supported that can't be inverted and has no readable prior
  state (from docs/design/0.2.0-decisions.md §2, high).

## Consequences

- `Op` carries thirteen variants: the nine a journal carries, plus `Remove`,
  `RemoveDir`, `RestoreBackup` and `Nothing`, which are what an inverse is (from
  https://github.com/meowshed/meowctl/pull/87, high).
- Repairing three `v0.1.0` inverse defects needed new journal fields, `created`
  and `until`, which `v0.1.0` ignores as unknown, so a record still replays
  there (from https://github.com/meowshed/meowctl/pull/59, high).
- `Op::Download` writes bytes somebody else fetched, because an operation that
  reached the network couldn't be applied against an in-memory filesystem (from
  https://github.com/meowshed/meowctl/pull/59, high). Taking the bytes as input
  is what keeps `Download` inside the `FileSystem`, so REQ-2002 and REQ-2102
  hold for it too (from those requirements, high).

## How I will know it was realised

1. `crates/meowctl-ops/tests/inverses.rs` holds a `proptest!` block and holds
   REQ-2001 and REQ-2017 (from crates/meowctl-ops/tests/inverses.rs, high).
2. `crates/meowctl-ctx/tests/surface.rs` checks a mutation is journalled before
   it's applied and holds REQ-2813 (from
   https://github.com/meowshed/meowctl/pull/70 and
   crates/meowctl-ctx/tests/surface.rs, high).

## What this does not settle

- Whether every variant's inverse is correct; ADR-0025 records the property test
  that checks it.
- The source recorded no alternative beyond the `v0.1.0` arrangement; the
  trade-off table also rejects an open trait, for the reason in REQ-2001 (from
  docs/design/0.2.0-decisions.md §2 and
  docs/design/0.2.0-requirement-tradeoffs.md, high).
