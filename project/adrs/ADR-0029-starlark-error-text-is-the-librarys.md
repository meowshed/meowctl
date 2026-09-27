---
id: ADR-0029
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2251, REQ-2370]
supersedes: []
---

# 0029. Accept starlark-rust's error messages rather than translate them

## Decision

Runtime error text is `starlark-rust`'s own, not translated to match
`go.starlark.net` (from docs/design/0.2.0-decisions.md §3.11, high). The
decision constrains `docs/spec/starlark.md` without being a single requirement
(from docs/design/0.2.0-decisions.md, high). REQ-2370 now states the exemption,
and REQ-2251 fixes what the library's argument message has to carry; REQ-2240
constrains the position rather than the text, so it is a consequence here
(from docs/spec/starlark.md:200-203, high).

Once accepted, a user who reads a message from both binaries sees two wordings,
for example "Missing parameter `name` for call to `component`" against
"component: missing argument for name" (from docs/design/0.2.0-decisions.md
§3.11, high).

## Why

The message isn't an interface a component depends on, and `starlark-rust`'s
rendering is better: it carries a span and a caret where `v0.1.0` names a file
(from docs/design/0.2.0-decisions.md §3.11, high). A translation table would
track a dependency's internal strings across versions and be wrong the first
time the library adds a message (from docs/design/0.2.0-decisions.md §3.11,
high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Translate the messages to match `go.starlark.net` | A user sees one wording across both binaries | A mapping table tracks internal strings and breaks when the library adds one (from docs/design/0.2.0-decisions.md §3.11, high) |

## What it costs

Two wordings for one mistake across the binaries, and snapshots taken from the
Rust side (from docs/design/0.2.0-decisions.md §3.11, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.11, high).

## Consequences

- REQ-2240 is satisfied by the library rather than by work in `meowctl-starlark`
  (from docs/spec/starlark.md, high).

## How I will know it was realised

1. `crates/meowctl-starlark/tests/evaluation.rs` holds REQ-2251 and REQ-2240
   (from crates/meowctl-starlark/tests/evaluation.rs, high).
2. A test holds REQ-2370; none cites it yet, because onboarding added it.

## What this does not settle

- The source recorded no alternative beyond translating (from
  docs/design/0.2.0-decisions.md §3.11, high).
