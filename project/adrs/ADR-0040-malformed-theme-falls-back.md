---
id: ADR-0040
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3252]
supersedes: []
---

# 0040. Warn and fall back to the default theme on a malformed theme file

## Decision

A malformed theme file warns and falls back to the default, rather than failing
the command (from docs/design/0.2.0-decisions.md §3.23, high). REQ-3252 states
it (from docs/spec/tui.md, high).

Once accepted, a bad colour file never stops an apply.

## Why

Nobody's apply should stop over colours (from docs/design/0.2.0-decisions.md
§3.23, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Fail the command on a malformed theme file | The user sees the mistake at once | An apply would stop over colours (from docs/design/0.2.0-decisions.md §3.23, high) |

## What it costs

A user whose file is wrong gets the default colours and a warning, not an error
(from docs/design/0.2.0-decisions.md §3.23, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.23, high).

## Consequences

- The requirement was deferred while no theme loader existed, and became live
  when `theme.toml` shipped (from https://github.com/meowshed/meowctl/pull/87
  and https://github.com/meowshed/meowctl/pull/92, high).

## How I will know it was realised

1. `tests/theme.rs` and `crates/meowctl-tui/tests/rendering.rs` hold REQ-3252
   (from those files, high).

## What this does not settle

- The source recorded no alternative beyond failing (from
  docs/design/0.2.0-decisions.md §3.23, high).
