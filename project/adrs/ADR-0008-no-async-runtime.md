---
id: ADR-0008
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3571, REQ-1801]
supersedes: []
---

# 0008. Use ureq and rustls with no async runtime

## Decision

The network stack is `ureq` with `rustls`, and no tokio (from
docs/design/0.2.0-rust-rewrite.md §3 and docs/design/0.2.0-decisions.md §2,
high). It addresses REQ-3571, because `meowctl hook shell` runs on every shell
spawn and must start nothing it doesn't need, and REQ-1801, the one blocking
method that returns bytes, where a synchronous stack shows in the interface
(from docs/design/0.2.0-rust-rewrite.md:27-28 and docs/spec/net.md, high).
REQ-1812 holds for any HTTP stack, so the decision doesn't address it (from
REQ-1812, high).

Once accepted, `meowctl hook shell` starts without initialising a runtime.
Module fetches are sequential (from docs/design/0.2.0-decisions.md §2, high).

## Why

The network work is a handful of sequential tarball fetches, and `ureq` with
`rustls` keeps startup cheap and avoids pulling tokio through the whole tree
(from docs/design/0.2.0-rust-rewrite.md §3, high). `meowctl hook shell` runs on
every shell spawn, so a runtime initialised on every interactive shell is a cost
paid thousands of times a day to save seconds on a sync that happens weekly
(from docs/design/0.2.0-decisions.md §2, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| `reqwest` with tokio | The default choice for HTTP in Rust; brings connection reuse and concurrency for free | It initialises a runtime on every shell spawn to speed up a weekly sync (from docs/design/0.2.0-decisions.md §2, high) |

## What it costs

Module fetches are sequential: a configuration with twenty modules pays twenty
round trips in series where a concurrent fetch would overlap them (from
docs/design/0.2.0-decisions.md §2, high).

## What would reverse it

- A configuration grows large enough that sync latency becomes the complaint,
  measured rather than assumed; the fix then is concurrency in the fetch, which
  doesn't need a runtime in the whole tree (from docs/design/0.2.0-decisions.md
  §2, high).

## Consequences

- `meowctl-net` wraps `ureq` behind the `Http` trait; ADR-0019 records that
  (from docs/design/0.2.0-decisions.md §2, high).
- `webpki-roots` ships the Mozilla CA list under CDLA-Permissive-2.0, which
  `deny.toml` allows with the reason (from
  https://github.com/meowshed/meowctl/pull/67, high).

## How I will know it was realised

1. The root `Cargo.toml` declares `ureq` with the `rustls` feature, and
   `rg tokio Cargo.lock` finds nothing (from Cargo.toml and Cargo.lock, high).

## What this does not settle

- How the fetch would become concurrent if the reversal condition occurs.
- The source recorded no alternative beyond `reqwest` with tokio (from
  docs/design/0.2.0-decisions.md §2, high).
