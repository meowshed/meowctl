---
id: ADR-0022
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1412]
supersedes: []
---

# 0022. Answer a dry-run read from what the dry run recorded

## Decision

`DryRunFs` answers a read of a path it recorded a write for with the content of
that write, not with what's on disk (from docs/spec/fs.md, high). REQ-1412
states it (from docs/design/0.2.0-decisions.md §3.3, high).

Once accepted, a hook that writes a file and reads it back takes the same branch
in a dry run as in the real run (from docs/design/0.2.0-decisions.md §3.3,
high).

## Why

A plan that predicts the wrong branch predicts the wrong run, which is what made
`v0.1.0`'s dry run something people stopped trusting (from
docs/design/0.2.0-decisions.md §3.3, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Pass every read through to the real filesystem | Simpler, and can't lie about disk | A hook that reads back its own write takes a different branch than the real run (from docs/design/0.2.0-decisions.md §3.3, high) |

## What it costs

`DryRunFs` has to remember writes and removals and answer from them; one test
failed because the dry run correctly remembered a removal (from
https://github.com/meowshed/meowctl/pull/57, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.3, high).

## Consequences

- A dry run also predicts failure where it can see the cause; ADR-0023 records
  that.

## How I will know it was realised

1. `crates/meowctl-fs/tests/dry_run.rs`,
   `crates/meowctl-fs/tests/conformance.rs` and `tests/dry_run.rs` hold REQ-1412
   (from those files, high).

## What this does not settle

- The source recorded no alternative beyond passing reads through (from
  docs/design/0.2.0-decisions.md §3.3, high).
