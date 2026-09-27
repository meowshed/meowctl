---
id: ADR-0033
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3412]
supersedes: []
---

# 0033. Construct the command tree per invocation

## Decision

Commands are constructed per invocation, where `v0.1.0` holds
`var RootCmd = newRootCmd()` (from docs/design/0.2.0-rust-rewrite.md §2
defect #8 and docs/design/0.2.0-decisions.md §3.17, high). REQ-3412 states it
(from docs/spec/cli.md, high).

Once accepted, each test builds its own command tree.

## Why

A package-level tree is what makes tests share mutable state and depend on order
(from docs/design/0.2.0-decisions.md §3.17, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Keep `var RootCmd = newRootCmd()` | Convenient for tests that reach in and run a command | The convenience is what makes those tests share state and depend on order (from docs/design/0.2.0-decisions.md §3.17, high) |

## What it costs

Tests lose a shared tree to reach into (from docs/design/0.2.0-decisions.md
§3.17, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.17, high).

## Consequences

- The surface tests drive `clap` directly, constructing one tree per test (from
  https://github.com/meowshed/meowctl/pull/76, high).

## How I will know it was realised

1. `crates/meowctl-cli/tests/surface.rs` holds REQ-3412 (from
   crates/meowctl-cli/tests/surface.rs, high).

## What this does not settle

- The source recorded no alternative beyond the package-level tree (from
  docs/design/0.2.0-decisions.md §3.17, high).
