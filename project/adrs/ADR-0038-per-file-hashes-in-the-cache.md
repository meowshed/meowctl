---
id: ADR-0038
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2430, REQ-2444]
supersedes: []
---

# 0038. Record per-file hashes in the module cache, not in the lock

## Decision

Every extracted file is verified against a per-file hash before it's evaluated,
and the hashes are recorded in the cache beside `v0.1.0`'s own `.sri` sidecar,
not in `deps.lock`'s `files` table (from docs/design/0.2.0-decisions.md §3.22,
high). REQ-2430 and REQ-2444 state it (from docs/spec/module.md, high).

Once accepted, anything that writes to `~/.cache` can no longer change what a
hook runs unnoticed (from https://github.com/meowshed/meowctl/pull/67, high).
The lock's `files` table stays declared and unwritten (from docs/spec/config.md,
high).

## Why

`v0.1.0` and `v0.2.0` share a machine during the cutover, and `v0.1.0`'s lock
writer replaces a module entry wholesale, so hashes in the lock would vanish on
its first run; filling the table would also end the byte-identical lock (from
docs/design/0.2.0-decisions.md §3.22, high). The question the hashes answer is
whether the cache still holds what was extracted, which is a question about the
cache (from docs/design/0.2.0-decisions.md §3.22, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Fill in `deps.lock`'s `files` table, which `v0.1.0` declares and never writes | The lock is the authority for what a module resolved to, and a hash there is reviewed in a diff | `v0.1.0` would erase the hashes on its next run, and the byte-identical lock would end (from docs/design/0.2.0-decisions.md §3.22, high) |

## What it costs

The hashes aren't reviewed in a diff, because they live in the cache (from
docs/design/0.2.0-decisions.md §3.22, high).

## What would reverse it

- `v0.1.0` is no longer installed anywhere: then the lock is the better home and
  the table is already there (from docs/design/0.2.0-decisions.md §3.22, high).

## Consequences

- The schema keeps the `files` table because a lock either binary wrote has to
  round-trip through the other (from
  https://github.com/meowshed/meowctl/pull/68, high).

## How I will know it was realised

1. `crates/meowctl-module/tests/fetching.rs` holds REQ-2430 and REQ-2444 (from
   crates/meowctl-module/tests/fetching.rs, high).

## What this does not settle

- Whether the reversal condition is met now that the Go tree is gone; the
  trade-off table says no such decision has been taken (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).
