---
id: ADR-0037
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1222, REQ-1223]
supersedes: []
---

# 0037. Fill the lock's meta table, and leave meta out of the byte comparison

## Decision

`deps.lock`'s `meta` table records the version that wrote the file and an RFC
3339 timestamp, and `meta` is excluded from the byte-identical comparison with
`v0.1.0` (from docs/design/0.2.0-decisions.md §3.21, high). REQ-1222 and
REQ-1223 state it (from docs/spec/config.md, high).

Once accepted, a user can see which binary last wrote their lock. A lock
`v0.2.0` writes differs from `v0.1.0`'s in two lines (from
docs/design/0.2.0-decisions.md §3.21, high).

## Why

The fields exist to say which binary last wrote the file, and that matters at a
cutover where two binaries share a configuration directory, which is the
situation this rewrite creates (from docs/design/0.2.0-decisions.md §3.21,
high). Everything a run depends on stays inside the comparison; the table meant
to disagree is the only thing outside it (from docs/design/0.2.0-decisions.md
§3.21, high). Nothing in `v0.1.0` ever sets either field (from
https://github.com/meowshed/meowctl/pull/68, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Leave `meta.generated-by` and `meta.updated-at` empty, as every `v0.1.0` lock does | Keeps the byte-identical lock, the strongest evidence the rewrite has, whole | It keeps two declared fields dead (from docs/design/0.2.0-decisions.md §3.21, high) |

## What it costs

The byte-identical lock loses two lines of its evidence (from
docs/design/0.2.0-decisions.md §3.21, high).

## What would reverse it

- `meta` stops being written (from docs/design/0.2.0-requirement-tradeoffs.md,
  high).

## Consequences

- The timestamp arrives as a value rather than being read from the clock, so it
  can be tested (from https://github.com/meowshed/meowctl/pull/75, high).

## How I will know it was realised

1. `crates/meowctl-module/tests/syncing.rs` produces the lock `v0.1.0` produced,
   byte for byte outside `meta`, and holds REQ-1222 and REQ-1223 (from
   https://github.com/meowshed/meowctl/pull/68 and
   crates/meowctl-module/tests/syncing.rs, high).

## What this does not settle

- The source recorded no alternative beyond leaving the fields empty (from
  docs/design/0.2.0-decisions.md §3.21, high).
