---
id: ADR-0041
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1210]
supersedes: []
---

# 0041. Give meowctl-config the starlark_syntax parser as an external dependency

## Decision

`meowctl-config` depends on `starlark_syntax` to edit Starlark files, an
external dependency of the same shape as `toml`, and not an edge to
`meowctl-starlark` (from https://github.com/meowshed/meowctl/pull/63, high).
Reading a `deps.mod` stays evaluation, which `meowctl-starlark` does, and
REQ-1210 says which of parsing and evaluating happens where (from
https://github.com/meowshed/meowctl/pull/63 and docs/spec/config.md, high).

Once accepted, editing and evaluation live in different crates, each with only
what it needs (from https://github.com/meowshed/meowctl/pull/63, high).

## Why

Editing needs the syntax tree, evaluation needs the builtins and the
accumulator, and neither needs the other's (from
https://github.com/meowshed/meowctl/pull/63, high). The design asserts three
boundaries and the crate table is a list, not a total order, so a dependency on
the `starlark` crates is a design question rather than a rule break (from
docs/design/0.2.0-rust-rewrite.md §3 and
https://github.com/meowshed/meowctl/pull/63, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Edit through `meowctl-starlark`, an edge between workspace crates | One crate names the dialect | Editing would carry the evaluator's builtins and accumulator it doesn't need, and the edge was first misread as a violation (from https://github.com/meowshed/meowctl/pull/63, high) |

## What it costs

The dialect is named in two crates; each says so and points at the other, and a
test runs one file through both, because a file the editor accepts and the
evaluator rejects would let `meowctl add` succeed on a configuration that then
fails to apply (from https://github.com/meowshed/meowctl/pull/63, high).

## What would reverse it

- The source names no reversal condition (from
  https://github.com/meowshed/meowctl/pull/63, high).

## Consequences

- A note under §3's crate table says the table is a list and names the three
  boundaries as the only ordering (from docs/design/0.2.0-rust-rewrite.md §3,
  high).

## How I will know it was realised

1. `crates/meowctl-config/Cargo.toml` declares `starlark_syntax`, and
   `crates/meowctl-config/tests/editing.rs` and
   `crates/meowctl-config/tests/parity.rs` hold REQ-1210 (from those files,
   high).

## What this does not settle

- The trade-off table's alternative for REQ-1210, a small parser of our own, is
  a different question and belongs to that requirement (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).
- The source recorded no alternative beyond an edge to `meowctl-starlark` (from
  https://github.com/meowshed/meowctl/pull/63, high).
