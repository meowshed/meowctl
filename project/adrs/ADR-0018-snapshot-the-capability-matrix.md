---
id: ADR-0018
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3280, REQ-3281]
supersedes: []
---

# 0018. Snapshot every sink across the whole capability matrix

## Decision

Snapshot tests with `insta` run a recorded event stream through each sink across
every cell of the capability matrix: motion, colour depth, glyph tier and width
(from docs/design/0.2.0-rust-rewrite.md §4, high). REQ-3281 requires the matrix
and REQ-3280 makes it possible, because a sink needs no terminal (from
docs/spec/tui.md and docs/design/0.2.0-decisions.md §3.23, high).

Once accepted, a tier nobody thought about can't ship untested (from
https://github.com/meowshed/meowctl/pull/71, high).

## Why

The stream is a fixture, so the tests need no terminal, no subprocess and no
filesystem, which is what makes enumerating the matrix feasible rather than
spot-checking two or three cases as the Go tests do (from
docs/design/0.2.0-rust-rewrite.md §4, high). Checking a few cases by hand is how
a tier nobody thought about ships broken (from
https://github.com/meowshed/meowctl/pull/71, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Spot-check a handful of cases, as `v0.1.0` does | Fewer snapshots to review | The untested cells are where the degradation bugs are (from docs/design/0.2.0-decisions.md §3.23 and docs/design/0.2.0-requirement-tradeoffs.md, high) |

## What it costs

It's affordable only because the sinks are testable from a fixture stream (from
docs/design/0.2.0-decisions.md §3.23, high). Escapes are rendered as `<up>`,
`<erase>` and so on so a person can review the snapshot (from
https://github.com/meowshed/meowctl/pull/71, high).

## What would reverse it

- The matrix grows enough that the snapshots dominate the suite (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- `insta` is a test dependency (from docs/design/0.2.0-rust-rewrite.md §3,
  high).

## How I will know it was realised

1. `crates/meowctl-tui/tests/live.rs` calls `insta::assert_snapshot!` over four
   colour depths, both glyph tiers and two widths, and holds REQ-3281 (from
   https://github.com/meowshed/meowctl/pull/71 and
   crates/meowctl-tui/tests/live.rs, high).

## What this does not settle

- Whether the snapshots should come from the Go binary, as §5 first planned for
  rendering drift; they come from the Rust side (from
  docs/design/0.2.0-rust-rewrite.md §5 and docs/design/0.2.0-decisions.md §3.11,
  medium).
- The source recorded no alternative beyond spot checks (from
  docs/design/0.2.0-decisions.md §3.23, high).
