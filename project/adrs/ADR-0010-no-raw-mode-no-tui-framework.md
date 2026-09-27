---
id: ADR-0010
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3204]
supersedes: []
---

# 0010. Render with terminal primitives only, and never take raw mode

## Decision

Terminal output uses `terminal_size` and escape sequences written by hand, and
no TUI framework (from Cargo.toml:39 and crates/meowctl-tui/Cargo.toml:15,
high). The plan named `anstyle`, `anstream`, `terminal-size` and
`unicode-width`, and only `terminal_size` became a dependency; the other three
appear in `Cargo.lock` only as transitive dependencies (from
docs/design/0.2.0-rust-rewrite.md:153 and Cargo.lock, high). No sink
puts the terminal into raw mode, and a full-screen TUI, a component browser, or
anything else that needs raw mode is out (from docs/design/0.2.0-rust-rewrite.md
§4 and §8, high). REQ-3204 states it (from docs/spec/tui.md, high). The live
sink writes only the portable control subset: cursor-up, erase-line, hide and
show cursor, and a synchronised-output pair; no alternate screen, no scroll
region, no cursor save and restore (from
https://github.com/meowshed/meowctl/pull/71, high).

Once accepted, a hook can hand the real terminal to an interactive command.
What doesn't work yet: without `unicode-width`, the live sink counts
characters, so a wide character can make a row wrap; BUG-0021 records it (from
crates/meowctl-tui/src/live.rs:458-466, medium).

## Why

Hooks shell out to commands that need the real terminal, and the never-raw-mode
rule is load-bearing, not a limitation to be lifted (from
docs/design/0.2.0-rust-rewrite.md §8, high). The excluded sequences are where
terminal support actually diverges (from
https://github.com/meowshed/meowctl/pull/71, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| A full-screen TUI or component browser | A richer interface | It needs raw mode, and hooks need the real terminal (from docs/design/0.2.0-rust-rewrite.md §8, high) |
| Take raw mode for a better renderer | A better renderer | Hooks shell out to commands that need the real terminal (from docs/design/0.2.0-requirement-tradeoffs.md, high) |

## What it costs

The live renderer is limited to redrawing a region with cursor-up and
erase-line, so it can't own the screen (from
https://github.com/meowshed/meowctl/pull/71, high). The terminal dependencies
port the `v0.1.0` design system rather than adopting a framework's (from
docs/design/0.2.0-rust-rewrite.md §3, high).

## What would reverse it

- Hooks stop being able to run interactive commands (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- Terminal ownership is handed over by events; ADR-0012 records that (from
  docs/design/0.2.0-rust-rewrite.md §4, high).
- A narrow terminal truncates rather than wraps, because a wrapped line breaks
  the next frame's arithmetic (from https://github.com/meowshed/meowctl/pull/71,
  high).

## How I will know it was realised

1. `crates/meowctl-tui/tests/live.rs` asserts none of the forbidden sequences is
   ever written and holds REQ-3204 (from
   https://github.com/meowshed/meowctl/pull/71 and
   crates/meowctl-tui/tests/live.rs, high).
2. `rg 'crossterm|ratatui' Cargo.lock` finds nothing (from Cargo.lock, high).
3. The root `Cargo.toml` declares `terminal_size` and none of `anstyle`,
   `anstream` or `unicode-width` (from Cargo.toml, high).

## What this does not settle

- Whether a future 0.3.0 feature could use an alternate screen for something
  that runs no hook; the source rules out only raw mode (from
  docs/design/0.2.0-rust-rewrite.md §8, low).
