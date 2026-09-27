---
id: ADR-0014
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1042, REQ-1043]
supersedes: []
---

# 0014. Carry everything worth printing as an event variant, and unstructured text only as Message with a severity

## Decision

Free-form `Log(format, args)` is gone: anything worth printing is an event
variant, unstructured text goes through `Message { severity, text }`, and the
severity decides whether it lands on stdout, stderr or nowhere (from
docs/design/0.2.0-rust-rewrite.md §4, high). REQ-1042 and REQ-1043 state it
(from docs/spec/common.md, high).

Once accepted, no event carries a rendered string, a colour, a glyph or a width.

## Why

`Writer.Log(format string, args ...any)` formats at the call site, so nothing
downstream can restructure it (from docs/design/0.2.0-rust-rewrite.md §2
defect #12, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Free-form `Log(format, args)` at the call site, as `v0.1.0` does | Any text anywhere, with no variant to add | Output formatted at the call site can't be restructured, which is why nothing had a machine-readable form (from docs/design/0.2.0-rust-rewrite.md §2 and §4, high) |

## What it costs

Anything a sink renders differently has to become its own variant; the cost
docs/design/0.2.0-decisions.md §2 gives for the event stream applies: a sink
wanting a detail the engine doesn't emit waits for the event to be added (from
docs/design/0.2.0-decisions.md §2, high).

## What would reverse it

- A case appears that genuinely has no structure and no severity (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- `print_stdout` and `print_stderr` are denied outside `meowctl-tui` by the
  workspace lints (from CLAUDE.md, high).

## How I will know it was realised

1. `crates/meowctl-common/src/event.rs` defines `Message` with a severity and
   holds REQ-1042 and REQ-1043 in its unit tests (from
   crates/meowctl-common/src/event.rs, medium).

## What this does not settle

- The source recorded no alternative beyond `v0.1.0`'s `Log` (from
  docs/design/0.2.0-rust-rewrite.md §4, high).
