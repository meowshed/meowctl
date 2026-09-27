---
id: ADR-0003
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3201, REQ-3202, REQ-3203, REQ-3210]
supersedes: []
---

# 0003. Redesign terminal output rather than port it, keeping v0.1.0's vocabulary

## Decision

Rendering is the one surface `v0.2.0` redesigns rather than ports (from
docs/design/0.2.0-rust-rewrite.md §1, high). Three ideas from
`internal/tui/theme.go` survive verbatim: one symbol vocabulary across every
command, never more than two levels of indent, and colour that is always
redundant to the wording; so does the rule that the live renderer never takes
raw mode (from docs/design/0.2.0-rust-rewrite.md §4, high). REQ-3201, REQ-3202
and REQ-3203 state the three ideas, and REQ-3210 states the sinks that replace
the old shape (from docs/spec/tui.md, high).

Once accepted, output is a set of sinks over an event stream. The redesign is
bounded by what §4 names: a full-screen TUI, a component browser, or anything
requiring raw mode stays out (from docs/design/0.2.0-rust-rewrite.md §8, high).

## Why

Terminal output is exempt from parity because it isn't an interface anything
depends on, and because the `v0.1.0` design leaks into the engine in a way the
rewrite has to fix anyway (from docs/design/0.2.0-rust-rewrite.md §1, high). Two
facts drive it: the engine renders, because `buildRunner` takes a `tui.Writer`,
and output is text at the point of production, because
`Log(format string, args ...any)` formats at the call site (from
docs/design/0.2.0-rust-rewrite.md §4, high). The vocabulary survives because
it's the reason the output is readable today (from
docs/design/0.2.0-rust-rewrite.md §4, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Port the output under the parity constraint | One constraint for the whole product, and the corpus could compare rendered text | It would port the engine holding a renderer (defect #11) and text-only output (defect #12) (from docs/design/0.2.0-rust-rewrite.md §4, high) |

## What it costs

The compatibility corpus couldn't compare rendered output, so a reworded status
line passed it; only `shell` stdout and `doctor --json` were compared byte for
byte at first (from https://github.com/meowshed/meowctl/pull/55, high).
`doctor --json` then left the comparison too, because holding its old shape
would hold the thing the redesign removes (from
https://github.com/meowshed/meowctl/pull/76, high). Rendering is held by
snapshots alone, because mutation testing stops at the rendering arithmetic,
where a width that rounds differently isn't a defect anybody can name (from
https://github.com/meowshed/meowctl/pull/94, high).

## What would reverse it

- A program is found that reads rendered or `--format json` output as an
  interface, as a shell reads `hook shell` stdout; a consumer of `v0.1.0`'s
  `doctor --json` shape is the concrete case (from
  docs/design/0.2.0-rust-rewrite.md:20-23 and
  https://github.com/meowshed/meowctl/pull/76, medium).
- For the raw-mode rule, hooks stop running interactive commands (from
  docs/design/0.2.0-requirement-tradeoffs.md:348, high).

## Consequences

- `meowctl-tui` becomes a redesign rather than a port, delivered as M7 (from
  docs/design/0.2.0-rust-rewrite.md §6, high).
- Mutation testing stops at the rendering arithmetic in `meowctl-tui`, where a
  width that rounds differently isn't a defect anybody can name (from
  https://github.com/meowshed/meowctl/pull/94, high).
- ADR-0010 to ADR-0018 record the parts of the redesign.

## How I will know it was realised

1. A test strips the escapes from a coloured render and asserts it equals the
   monochrome render, and another allows indents of only zero, two and six;
   `crates/meowctl-tui/tests/rendering.rs` holds REQ-3203 and REQ-3210 (from
   https://github.com/meowshed/meowctl/pull/64 and
   crates/meowctl-tui/tests/rendering.rs, high).
2. `internal/tui` is gone and `crates/meowctl-tui/src` holds `LiveSink`,
   `PlainSink`, `JsonSink` and `ShellSink` (from crates/meowctl-tui/src/live.rs
   and crates/meowctl-tui/src/sink.rs, high).

## What this does not settle

- Whether `--format json` output becomes a compatibility surface; ADR-0013
  records that it does once anybody parses it (from
  docs/design/0.2.0-decisions.md §3.15, high).
- The source recorded no alternative beyond porting the output (from
  docs/design/0.2.0-rust-rewrite.md §1, high).
