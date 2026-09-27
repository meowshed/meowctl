---
id: ADR-0025
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2004]
supersedes: []
---

# 0025. Check apply-then-undo with a property test over every Op variant

## Decision

Applying an `Op` and then its inverse restores the filesystem, for every
variant, and a property test over `MemFs` checks it (from docs/spec/ops.md,
high). REQ-2004 states it (from docs/design/0.2.0-decisions.md §3.7, high).

Once accepted, a variant added later is checked without anybody writing a test
for it. The property strategy can't reach a state where the operation already
happened, so REQ-2005 needs its own cases (from
https://github.com/meowshed/meowctl/pull/85, high).

## Why

The failure it catches is the one nobody writes a test for: a variant added
later whose inverse is subtly wrong (from docs/design/0.2.0-decisions.md §3.7,
high). It found three defects `v0.1.0` has: `link_file` copied its backup
instead of moving it, `append_file` left a zero-byte file, and `mkdir` removed
one directory of a `mkdir -p` chain (from
https://github.com/meowshed/meowctl/pull/59, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Test the inverses that seem risky, as `v0.1.0` does | A per-variant suite is easier to read | It misses the variant nobody thought risky (from docs/design/0.2.0-decisions.md §3.7, high) |

## What it costs

Setup, and a run that can be slow enough to notice (from
docs/design/0.2.0-decisions.md §3.7, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.7, high).

## Consequences

- Each repair needed a field in the journal payload (from
  https://github.com/meowshed/meowctl/pull/59, high).

## How I will know it was realised

1. `crates/meowctl-ops/tests/inverses.rs` holds a `proptest!` block and holds
   REQ-2004 (from crates/meowctl-ops/tests/inverses.rs, high).

## What this does not settle

- The source recorded no alternative beyond testing the risky inverses (from
  docs/design/0.2.0-decisions.md §3.7, high).
