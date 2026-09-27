---
id: ADR-0046
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1606]
supersedes: []
---

# 0046. Compute the PATHEXT order as a pure function of the name and the variable

## Decision

The order in which a name is tried on Windows is a pure function of the name and
the `PATHEXT` string, tested on every platform: the bare name first, then each
extension in the order the variable gives, empty entries skipped, entries
trimmed, and `.COM;.EXE;.BAT;.CMD` as the fallback (from
https://github.com/meowshed/meowctl/pull/88, high). REQ-1606 states the
behaviour (from docs/spec/exec.md, high).

Once accepted, the behaviour is tested on Linux, where CI runs.

## Why

The order was computed inside a `#[cfg(windows)]` function that read `PATHEXT`
itself, so a test would have had to mutate the process environment, which is
`unsafe` in edition 2024 while the workspace denies `unsafe_code` (from
https://github.com/meowshed/meowctl/pull/88, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Keep the `#[cfg(windows)]` function that reads `PATHEXT` itself | No change to working code | Testing it needs `unsafe` environment mutation, which the workspace denies (from https://github.com/meowshed/meowctl/pull/88, high) |

## What it costs

The code had to move before the requirement could be tested (from
https://github.com/meowshed/meowctl/pull/88, high).

## What would reverse it

- The source names no reversal condition (from
  https://github.com/meowshed/meowctl/pull/88, high).

## Consequences

- CI runs on Linux only, so `mise run check-windows` is what remains for the
  `cfg(windows)` caller (from https://github.com/meowshed/meowctl/pull/91 and
  CLAUDE.md, high).

## How I will know it was realised

1. `crates/meowctl-exec/src/lib.rs` holds REQ-1606 in its unit tests (from
   crates/meowctl-exec/src/lib.rs, medium).

## What this does not settle

- The trade-off table's alternative for REQ-1606, trying the bare name only, is
  a different question and belongs to that requirement (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).
