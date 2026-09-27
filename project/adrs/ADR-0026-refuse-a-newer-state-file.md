---
id: ADR-0026
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1241]
supersedes: []
---

# 0026. Refuse to overwrite a state.toml written by a newer meowctl

## Decision

A `state.toml` whose `schema_version` is higher than this build understands is
reported as written by a newer meowctl and isn't overwritten (from
docs/spec/config.md, high). REQ-1241 states it (from
docs/design/0.2.0-decisions.md §3.8, high).

Once accepted, an older binary stops rather than rewriting a newer file.

## Why

Ignoring the field means the older binary rewrites the file and drops whatever
it didn't understand, which is data loss with no message; refusing is visible
and recoverable, and the loss is neither (from docs/design/0.2.0-decisions.md
§3.8, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Ignore `schema_version`, as `v0.1.0` does | An older binary keeps working on a machine that ran a newer one | It loses data with no message (from docs/design/0.2.0-decisions.md §3.8, high) |

## What it costs

It blocks an older binary on a machine that ran a newer one, a real
inconvenience during the rewrite when two binaries share a config directory
(from docs/design/0.2.0-decisions.md §3.8, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.8, high).

## Consequences

- Constructing a `Phase` from a string that names no phase fails, because a
  newer `state.toml` can contain one (from docs/spec/common.md, high).

## How I will know it was realised

1. `crates/meowctl-config/tests/parity.rs` holds REQ-1241 (from
   crates/meowctl-config/tests/parity.rs, high).

## What this does not settle

- The source recorded no alternative beyond ignoring the field (from
  docs/design/0.2.0-decisions.md §3.8, high).
