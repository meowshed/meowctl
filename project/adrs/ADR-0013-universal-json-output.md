---
id: ADR-0013
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3210, REQ-3232, REQ-3406]
supersedes: []
---

# 0013. Make --format json available on every command, as a sink over the event stream

## Decision

Every command emits the event stream as JSON under `--format json`, through
`JsonSink`, one object per event per line (from
docs/design/0.2.0-rust-rewrite.md §4 and docs/design/0.2.0-decisions.md §3.15,
high). Text, JSON and live rendering are sinks over one stream: `LiveSink`,
`PlainSink` and `JsonSink`, with `ShellSink` added later for `meowctl hook`
(from docs/design/0.2.0-rust-rewrite.md §4 and docs/spec/tui.md, high).
REQ-3406, REQ-3232 and REQ-3210 state it (from docs/spec/cli.md and
docs/spec/tui.md, high).

Once accepted, `doctor --json` stops being a special case. Its `v0.1.0` shape, a
hand-written list of checks, isn't kept (from
https://github.com/meowshed/meowctl/pull/76, high).

## Why

Universal JSON costs one sink rather than one flag per command, and the compat
corpus needed structured output to diff (from docs/design/0.2.0-decisions.md
§3.15, high). In `v0.1.0` `doctor --json` is a hand-written special case and
nothing else has a machine-readable form (from docs/design/0.2.0-rust-rewrite.md
§2 defect #12, high). Holding the old `doctor --json` shape byte-exact would
hold the thing the redesign removes (from
https://github.com/meowshed/meowctl/pull/76, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Port `doctor --json` as the special case it is | Keeps `v0.1.0`'s JSON body byte for byte | One flag per command, and nothing else gains a machine-readable form (from docs/design/0.2.0-decisions.md §3.15 and https://github.com/meowshed/meowctl/pull/76, high) |

## What it costs

Every event has to be worth serializing, and the stream stays a compatibility
surface once anybody parses it (from docs/design/0.2.0-decisions.md §3.15,
high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.15, high).

## Consequences

- `--json` stays accepted as an alias on `doctor` and `status` (from
  docs/spec/cli.md, high).
- The compat corpus stopped comparing `doctor --json`, and `shell` stayed
  byte-exact (from https://github.com/meowshed/meowctl/pull/76, high).

## How I will know it was realised

1. `crates/meowctl-cli/tests/surface.rs` holds REQ-3406, and
   `crates/meowctl-tui/tests/rendering.rs` holds REQ-3232 and REQ-3210 (from
   those files, high).
2. `crates/meowctl-tui/src/sink.rs` defines `JsonSink`, `PlainSink` and
   `ShellSink`, and `crates/meowctl-tui/src/live.rs` defines `LiveSink` (from
   those files, high).

## What this does not settle

- Whether the JSON event shape is versioned for the programs that parse it.
- The source recorded no alternative beyond the special case (from
  docs/design/0.2.0-decisions.md §3.15, high).
