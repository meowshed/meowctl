---
id: ADR-0042
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1223]
supersedes: []
---

# 0042. Reproduce v0.1.0's TOML layout byte for byte rather than emit the toml crate's

## Decision

The lock writer reproduces `BurntSushi/toml`'s layout: two spaces per level of
nesting, a blank line before each top-level table, map keys sorted and struct
fields in declaration order, a key quoted only when it has to be, an empty map
emitted as nothing, and a datetime unquoted (from
https://github.com/meowshed/meowctl/pull/60, high). REQ-1223 is the requirement
it serves (from docs/spec/config.md, high).

Once accepted, a lock either binary writes shows no whitespace change in a
user's `git diff`.

## Why

Rust's `toml` crate emits valid TOML that looks nothing like
`BurntSushi/toml`'s, and neither is wrong; but two binaries share a
configuration directory during the rewrite, and a lock that differs only in
whitespace still shows up as a change in every `git diff` a user takes of their
dotfiles (from https://github.com/meowshed/meowctl/pull/60, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Emit the `toml` crate's own layout | Correct TOML with no custom writer | Every lock rewrite would show as a change in the user's `git diff` (from https://github.com/meowshed/meowctl/pull/60, high) |

## What it costs

A hand-written layout to own, checked against a fixture `lock.Write` produced
with every field populated, because a round trip through our own reader and
writer proves only that the two agree with each other (from
https://github.com/meowshed/meowctl/pull/60, high).

## What would reverse it

- The source names no reversal condition (from
  https://github.com/meowshed/meowctl/pull/60, high).

## Consequences

- `pkgs.lock` and `pkgs.local.lock` aren't byte-compared, because `v0.1.0`
  writes them with `toml.NewEncoder` rather than by hand (from
  https://github.com/meowshed/meowctl/pull/84, high).

## How I will know it was realised

1. `crates/meowctl-config/tests/parity.rs` compares a written lock against the
   `v0.1.0` fixture and holds REQ-1223 (from
   https://github.com/meowshed/meowctl/pull/60 and
   crates/meowctl-config/tests/parity.rs, high).

## What this does not settle

- The source recorded no alternative beyond the `toml` crate's output (from
  https://github.com/meowshed/meowctl/pull/60, high).
