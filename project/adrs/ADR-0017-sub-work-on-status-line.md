---
id: ADR-0017
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3224]
supersedes: []
---

# 0017. Show sub-work on the component's own status line rather than a third indent

## Decision

The live sink shows sub-work, such as packages being installed, on the
component's own status line rather than by indenting further (from
docs/design/0.2.0-rust-rewrite.md §4, high). REQ-3224 states it (from
docs/spec/tui.md, high).

Once accepted, a component installing forty packages looks different from one
installing none, and output keeps two indent levels (from
docs/design/0.2.0-rust-rewrite.md §4, high).

## Why

The engine's unit of work is a component within a phase, and packages within a
component were invisible (from docs/design/0.2.0-rust-rewrite.md §4, high).
Putting them on the status line keeps the two-level rule intact (from
docs/design/0.2.0-rust-rewrite.md §4, high), and that rule is what makes output
scannable (from docs/design/0.2.0-decisions.md §3.23, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Indent sub-work as a third level | More detail per package | It breaks the two-indent rule that makes output scannable (from docs/design/0.2.0-decisions.md §3.23 and docs/design/0.2.0-requirement-tradeoffs.md, high) |

## What it costs

Detail: sub-work gets one status line rather than a line each (from
docs/design/0.2.0-decisions.md §3.23, high).

## What would reverse it

- Sub-work becomes important enough to justify a third level (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- The indent test allows only zero, two and six columns (from
  https://github.com/meowshed/meowctl/pull/64, high).

## How I will know it was realised

1. `crates/meowctl-tui/tests/live.rs` checks sub-work on the component's own
   status line and holds REQ-3224 (from
   https://github.com/meowshed/meowctl/pull/71 and
   crates/meowctl-tui/tests/live.rs, high).

## What this does not settle

- The source recorded no alternative beyond a third indent (from
  docs/design/0.2.0-decisions.md §3.23, high).
