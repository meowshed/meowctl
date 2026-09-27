---
id: ADR-0016
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2826, REQ-3260, REQ-3262, REQ-3263]
supersedes: []
---

# 0016. Prompt through an Interaction trait that fails when nobody can answer

## Decision

An `Interaction` trait with `confirm` and `ask` replaces `Printer.Confirm`,
with a non-interactive implementation that fails loudly rather than hanging
(from docs/design/0.2.0-rust-rewrite.md §4, high). REQ-3260 separates prompting
from rendering, REQ-3262 fails a prompt in a non-interactive session, and
REQ-2826 binds `ctx.prompt` to the trait, so no hook reads stdin (from
docs/spec/tui.md, docs/spec/ctx.md and docs/design/0.2.0-decisions.md §3.23,
high). REQ-3263 is `ask`, the free-text question `ctx.prompt` needs (from
https://github.com/meowshed/meowctl/pull/70, high).

Once accepted, a CI run that reaches a prompt stops with a message naming what
it wanted. No command and no `ctx` method offers a choice from a list, so the
trait has none (from crates/meowctl-tui/src/interaction.rs:33-52, high).

## Why

`Printer.Confirm` mixes input into an output type (from
docs/design/0.2.0-rust-rewrite.md §4, high), which made it hard to test and is
why its bug shipped (from https://github.com/meowshed/meowctl/pull/65, high). A
CI run that hits a prompt should die with a clear message, not block until
timeout (from docs/design/0.2.0-rust-rewrite.md §4, high). The non-interactive
session fails without reading, because reading would consume whatever the pipe
held and treat it as an answer (from
https://github.com/meowshed/meowctl/pull/65, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| `Printer.Confirm`, input inside the output type, as `v0.1.0` does | One type for the terminal | Input in an output type is hard to test, and its bug shipped (from docs/design/0.2.0-rust-rewrite.md §4 and https://github.com/meowshed/meowctl/pull/65, high) |
| A `select` method, a choice from a list, as the plan named | A command could offer a menu | Nothing in the frozen command or Starlark surface calls it, so it would be untested surface; deferred until a command or `ctx` method needs one (from docs/design/0.2.0-rust-rewrite.md:219 and docs/spec/ctx.md, high) |
| Block waiting for input in a non-interactive session | Never fails a run that a person could have answered | A CI run hangs until it times out (from docs/design/0.2.0-decisions.md §3.23 and docs/design/0.2.0-requirement-tradeoffs.md, high) |

## What it costs

A possible false failure, traded for a CI run that hangs until it times out
(from docs/design/0.2.0-decisions.md §3.23, high).

## What would reverse it

- A non-interactive default answer is wanted more than a failure (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- The question goes to stderr, so a piped stdout isn't corrupted (from
  https://github.com/meowshed/meowctl/pull/65, high).
- `--yes` maps to an `Always` implementation, wired by the CLI (from
  https://github.com/meowshed/meowctl/pull/65, high).

## How I will know it was realised

1. `crates/meowctl-tui/src/interaction.rs` defines `pub trait Interaction` and
   holds REQ-3260 and REQ-3262 in its unit tests (from
   crates/meowctl-tui/src/interaction.rs, medium).
2. `crates/meowctl-ctx/tests/surface.rs` holds REQ-2826 (from
   crates/meowctl-ctx/tests/surface.rs, high).

## What this does not settle

- Whether 0.3.0 adds a list choice; that is a decision for when a caller
  exists, because the Starlark API is frozen at `v0.1.0` (from CLAUDE.md,
  high).
- How a free-text question differs from a confirmation; REQ-3263 carries that.
