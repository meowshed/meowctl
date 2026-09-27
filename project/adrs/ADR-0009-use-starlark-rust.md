---
id: ADR-0009
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2211, REQ-2252]
supersedes: []
---

# 0009. Evaluate Starlark with the starlark-rust crate rather than an evaluator of our own

## Decision

`v0.2.0` runs the `starlark` crate, starlark-rust, rather than an evaluator
written in-house (from docs/design/0.2.0-decisions.md §2 and
docs/design/0.2.0-rust-rewrite.md §3, high). M0 ran against `starlark` 0.14.2
and found extension points for all five bindings the project needs (from
docs/design/0.2.0-rust-rewrite.md §5, high). REQ-2211 records the constraint the
crate imposes, that no Starlark value outlives its evaluation, and REQ-2252
records that the crate offers no step limit (from docs/spec/starlark.md, high).

Once accepted, the evaluator is a dependency. Runtime error text differs from
`go.starlark.net`; ADR-0029 records that choice.

## Why

Writing a Starlark evaluator is the whole project, not a part of it (from
docs/design/0.2.0-decisions.md §2, high). M0 answered every parity question:
`FileLoader` for the composite loader, `StarlarkValue` with `get_attr` for
`ctx`, `Evaluator::extra` for per-evaluation state, `eval_function` for calling
a hook, and `Error::span` for a diagnostic with a caret (from
docs/design/0.2.0-rust-rewrite.md §5, high). The alternatives the spike existed
to price aren't needed (from docs/design/0.2.0-rust-rewrite.md §5, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Write an evaluator in-house | The only way to guarantee message-for-message parity with `go.starlark.net` | Writing a Starlark evaluator is the whole project (from docs/design/0.2.0-decisions.md §2, high) |
| Vendor a patched starlark-rust | Fixes a dialect difference the upstream crate won't take | M0 found no difference that needed it (from docs/design/0.2.0-rust-rewrite.md §5, high) |

## What it costs

Dialect differences the project doesn't control, including error text, and a
crate whose API changes between minor versions: 0.14 moved `Heap` behind a
lifetime and scoped `Module` to a closure, both of which changed how `ctx` is
written (from docs/design/0.2.0-decisions.md §2, high). Four crates in its
dependency tree carry unmaintained advisories, recorded in `deny.toml` because
the alternative was to stop using the evaluator (from
https://github.com/meowshed/meowctl/pull/61, high). It pulls in `blake3`, whose
build script needs a C toolchain, so the local Windows cross-check no longer
builds the whole workspace (from https://github.com/meowshed/meowctl/pull/61,
high).

## What would reverse it

- A dialect difference a component can observe that the crate won't accept a fix
  for; M0 found none (from docs/design/0.2.0-decisions.md §2, high).

## Consequences

- `Module::with_temp_heap` scopes the heap to a closure, so the accumulator's
  owned-data rule is enforced by the compiler (from
  docs/design/0.2.0-rust-rewrite.md §5, high).
- `json` had to be supplied as a module, because starlark-rust's `json` is a
  function under the same name (from
  https://github.com/meowshed/meowctl/pull/61, high).
- A package-manager handler can't be held across evaluations, so `meowctl-pm`
  produces a `Call`; ADR-0043 records that (from
  https://github.com/meowshed/meowctl/pull/69, high).

## How I will know it was realised

1. `crates/meowctl-starlark/src/evaluator.rs` calls `Module::with_temp_heap`,
   and `crates/meowctl-starlark/tests/evaluation.rs` holds REQ-2211 (from those
   files, high).
2. `crates/meowctl-starlark/tests/stdlib.rs` evaluates the real `meowctl-stdlib`
   and `dotmeow` trees when they're checked out beside the repository, and skips
   otherwise, so a green run without them proves nothing about the corpus (from
   CLAUDE.md and crates/meowctl-starlark/tests/stdlib.rs, medium).

## What this does not settle

- How error text is handled; ADR-0029 records that (from
  docs/design/0.2.0-decisions.md §3.11, high).
- How an unbounded evaluation is stopped; REQ-2252 points at the second
  interrupt (from https://github.com/meowshed/meowctl/pull/87, high).
