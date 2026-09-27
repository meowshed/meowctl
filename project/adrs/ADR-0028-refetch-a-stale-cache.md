---
id: ADR-0028
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2442, REQ-2443]
supersedes: []
---

# 0028. Re-fetch a cached module whose files don't match, rather than fail

## Decision

A cached module whose files don't match the recorded hashes is re-fetched rather
than used or reported as an error (from docs/design/0.2.0-decisions.md §3.10,
high). A fully cached, verified lock resolves with no network (from
docs/design/0.2.0-decisions.md §3.10, high). REQ-2442 and REQ-2443 state it
(from docs/spec/module.md, high).

Once accepted, a cache truncated by a full disk or a killed process repairs
itself on the next run.

## Why

The lock is the authority and the cache is a copy (from
docs/design/0.2.0-decisions.md §3.10, high). Re-fetching is what a user wants
when the cache was truncated by a full disk or a killed process, which is the
common cause (from docs/design/0.2.0-decisions.md §3.10, high). Without offline
resolution, `meowctl hook shell` hangs on a plane (from
docs/design/0.2.0-decisions.md §3.10, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Treat a cache mismatch as an error | Surfaces tampering loudly | The common cause is truncation, and the lock already says what the module is (from docs/design/0.2.0-decisions.md §3.10, high) |

## What it costs

Tampering with the cache is repaired quietly rather than surfaced (inferred from
docs/design/0.2.0-decisions.md §3.10, low).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.10, high).

## Consequences

- Per-file hashes live beside the cache; ADR-0038 records where (from
  docs/design/0.2.0-decisions.md §3.22, high).

## How I will know it was realised

1. `crates/meowctl-module/tests/fetching.rs` holds REQ-2442 and REQ-2443 (from
   crates/meowctl-module/tests/fetching.rs, high).

## What this does not settle

- The source recorded no alternative beyond failing (from
  docs/design/0.2.0-decisions.md §3.10, high).
