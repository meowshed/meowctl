---
id: ADR-0019
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1803, REQ-1814, REQ-2443, REQ-1805]
supersedes: []
---

# 0019. Put every network request behind one Http trait in meowctl-net

## Decision

`meowctl-net` holds `trait Http` beside `meowctl-fs` and `meowctl-exec`, and
every request in the program goes through it (from
docs/design/0.2.0-decisions.md §2, high). It has `RealHttp`, `ScriptedHttp` and
`OfflineHttp` (from docs/design/0.2.0-rust-rewrite.md §3, high). REQ-2443,
REQ-1803 and REQ-1814 are the requirements it makes checkable (from
docs/design/0.2.0-decisions.md §2 and docs/spec/net.md, high).

Once accepted, the HTTPS rule is enforced in one place, and offline resolution
is proved by an implementation that can't make a request.

## Why

Two crates need it and neither may depend on the other: `meowctl-module` fetches
indexes and tarballs, `meowctl-ctx` implements `ctx.download`, and they sit side
by side under the engine, so a shared trait has to live below both (from
docs/design/0.2.0-decisions.md §2, high). `OfflineHttp` proves REQ-2443 by
construction, and REQ-1803's HTTPS rule is enforced in one place rather than at
each call (from docs/design/0.2.0-decisions.md §2, high). `v0.1.0` accepts any
scheme, so one `source` template in a registry index could downgrade every
module fetch to plaintext (from https://github.com/meowshed/meowctl/pull/67,
high). One `https_only` setting gives both REQ-1803 and REQ-1805, the refusal of
a redirect to plaintext (from https://github.com/meowshed/meowctl/pull/85,
high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| An HTTP client in `meowctl-module` and another for `ctx.download`, as `v0.1.0` does with three `*http.Client` fields | No extra crate and no extra parameter | The HTTPS rule would be enforced at each call, and offline resolution would rest on a promise (from docs/design/0.2.0-decisions.md §2, high) |

## What it costs

A fourteenth crate for one trait and three small implementations, and one more
parameter threaded from the binary; the TLS stack is the same either way (from
docs/design/0.2.0-decisions.md §2, high).

## What would reverse it

- A second consumer never appears, which would make it a crate wrapping one
  dependency for one caller (from docs/design/0.2.0-decisions.md §2, high).

## Consequences

- The response body is bounded at 64 MiB, where `io.ReadAll` had no bound (from
  https://github.com/meowshed/meowctl/pull/67, high).
- `meowctl-net` has no row in `CLAUDE.md`'s crate table, though §3 of the plan
  lists it (from CLAUDE.md and docs/design/0.2.0-rust-rewrite.md §3, high).

## How I will know it was realised

1. `crates/meowctl-module/tests/fetching.rs` holds REQ-2443, and
   `tests/errors.rs` refuses an `http://` `MEOWCTL_RELEASES` before anything is
   fetched, holding REQ-1803 (from those files and
   https://github.com/meowshed/meowctl/pull/93, high).
2. `crates/meowctl-net/src/offline.rs` holds REQ-1814 in its unit tests (from
   crates/meowctl-net/src/offline.rs, medium).

## What this does not settle

- The source recorded no alternative beyond one client per consumer (from
  docs/design/0.2.0-decisions.md §2, high).
