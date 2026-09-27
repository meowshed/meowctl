---
id: ADR-0015
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3250, REQ-3251]
supersedes: []
---

# 0015. Hold the palette as data that call sites name by role

## Decision

The Catppuccin palette stays the default, but as values rather than truecolor
SGR constants compiled into the binary; users can point at their own, roles stay
semantic so no call site names a colour, and downsampling stays automatic (from
docs/design/0.2.0-rust-rewrite.md §4, high). REQ-3250 and REQ-3251 state it
(from docs/spec/tui.md, high).

Once accepted, the palette changes in one place. The user's file shipped after
the rest of the redesign, in `theme.toml` (from
https://github.com/meowshed/meowctl/pull/92, high).

## Why

The source gives the redesign's reason for the whole of §4; for the theme it
states the outcome, a user can point at their own palette, as the gain (from
docs/design/0.2.0-rust-rewrite.md §4, high). A user whose terminal clashes with
Catppuccin otherwise has no recourse (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Truecolor SGR constants compiled into the binary, as `v0.1.0` does | No file to read or validate | A user can't change a colour, and call sites can name colours (from docs/design/0.2.0-rust-rewrite.md §4 and docs/design/0.2.0-requirement-tradeoffs.md, high) |

## What it costs

A file format to own, with its own failure paths: ADR-0040 records what a
malformed file does (from docs/design/0.2.0-decisions.md §3.23, high).

## What would reverse it

- The trade-off row for REQ-3250 reads "Never" (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- REQ-3250 was cut back to "the palette is one table" when no loader had been
  built, and restored when `theme.toml` shipped (from
  https://github.com/meowshed/meowctl/pull/87 and
  https://github.com/meowshed/meowctl/pull/92, high).

## How I will know it was realised

1. `tests/theme.rs` drives the binary with a user palette and holds REQ-3250;
   `crates/meowctl-tui/tests/rendering.rs` holds REQ-3251 (from those files,
   high).

## What this does not settle

- Where the file lives and how it's parsed; the requirements from
  `docs/spec/tui.md` that the theme file carries state that.
- The source recorded no alternative beyond compiled constants (from
  docs/design/0.2.0-rust-rewrite.md §4, high).
