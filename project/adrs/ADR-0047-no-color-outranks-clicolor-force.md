---
id: ADR-0047
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3246]
supersedes: []
---

# 0047. Let NO_COLOR win over CLICOLOR_FORCE

## Decision

`CLICOLOR_FORCE`, set to anything but `0`, turns colour back on for a
destination that isn't a terminal, and `NO_COLOR` still wins over it (from
docs/spec/tui.md and https://github.com/meowshed/meowctl/pull/88, high).
REQ-3246 states it (from docs/spec/tui.md, high).

Once accepted, a user who asked for no colour never gets it.

## Why

A user saying they want no colour anywhere beats a caller saying this particular
pipe can take it (from https://github.com/meowshed/meowctl/pull/88, high). The
variable had been implemented and changed behaviour with no requirement naming
it (from https://github.com/meowshed/meowctl/pull/88, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Let `CLICOLOR_FORCE` win | A caller that renders the output itself always gets colour | It overrides a user who asked for no colour anywhere (from https://github.com/meowshed/meowctl/pull/88, high) |

## What it costs

A caller that sets `CLICOLOR_FORCE` gets no colour on a machine where the user
set `NO_COLOR` (inferred from https://github.com/meowshed/meowctl/pull/88, low).

## What would reverse it

- The source names no reversal condition (from
  https://github.com/meowshed/meowctl/pull/88, high).

## Consequences

- REQ-3245 names `COLORTERM` and `TERM` as how a terminal reports its depth
  (from https://github.com/meowshed/meowctl/pull/88, high).

## How I will know it was realised

1. `crates/meowctl-tui/src/caps.rs` holds REQ-3246 in its unit tests (from
   crates/meowctl-tui/src/caps.rs, medium).

## What this does not settle

- The trade-off table's alternative for REQ-3246, never colouring a pipe, is a
  different question and belongs to that requirement (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).
