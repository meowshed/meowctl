---
id: vision
artifact: vision
status: live
revised: 2026-09-27
---

<!-- Written to the writing standard meow-prose ships: lead with the answer,
give each rule its reason in the same sentence, and show the failing case. -->

# meowctl

## What it is

meowctl is a dotfiles and developer environment manager powered by Starlark
(from README.md, high). You describe a machine as a set of components, each a
Starlark file that says what to install and what to link, and meowctl resolves
the modules those components come from, orders them by their declared
dependencies and runs them through the lifecycle phases (from README.md, high).
A failure rolls back what the run had done (from README.md, high).

Version 0.2.0, written in Rust, replaces the Go implementation released as
`v0.1.0` (from README.md, high). The Go code's problems were architectural, so a
mechanical port would have carried all of them across (from
docs/design/0.2.0-rust-rewrite.md, high). The repository names no competing tool
outside its own history, so this section compares meowctl only with `v0.1.0`.

## The problem

A configuration that applied under `v0.1.0` has to apply identically under
0.2.0, because users' `init.star`, `deps.mod`, lock files and `state.toml` are
already on their machines (from docs/design/0.2.0-rust-rewrite.md, high). Under
`v0.1.0`, a dry run could claim work the runner then skipped, because the
dry-run check was a branch repeated in every effectful method and one was missed
(from docs/design/0.2.0-rust-rewrite.md, high). Output was only ever text, so a
script had no machine-readable view of a run outside `doctor --json` (from
docs/design/0.2.0-rust-rewrite.md, high).

## Who it is for

| Audience | Wants | What they do today instead |
| -------- | ----- | -------------------------- |
| A developer who keeps dotfiles for several machines (from README.md, medium) | One command that brings a machine to what the configuration says, and a dry run that tells the truth (from README.md, high) | Not recorded in the repository |
| An author of components in `meowctl-stdlib` or a module registry (from README.md, high) | A Starlark API that stays stable across releases (from docs/design/0.2.0-rust-rewrite.md, high) | Not recorded in the repository |

## Quality goals

1. Compatibility with `v0.1.0`: the Starlark API, the command surface and every
   config and lock format stay compatible, byte for byte where the file is
   machine-written (from CLAUDE.md, high).
2. Startup cost, because `meowctl hook shell` runs on every shell spawn (from
   docs/design/0.2.0-rust-rewrite.md, high).
3. Every reversible effect has an inverse, so a failed run rolls back (from
   CLAUDE.md, high).
4. Testability: effects sit behind traits, so a test runs without the real
   filesystem, network or processes (from docs/design/0.2.0-rust-rewrite.md,
   high).

The documents state each goal but not their order; the order above ranks
compatibility first because CLAUDE.md calls a change that breaks it a
regression, and ranks the rest in the order the rewrite plan lists them (from
CLAUDE.md and docs/design/0.2.0-rust-rewrite.md, low).

## What it will not do

- New commands, changes to the Starlark API surface and config format changes
  wait for 0.3.0 (from docs/design/0.2.0-rust-rewrite.md, high).
- Parallel component execution, a daemon or server mode, and Windows support
  beyond what `v0.1.0` already builds wait for 0.3.0 (from
  docs/design/0.2.0-rust-rewrite.md, high).
- A full-screen interface, a component browser or anything requiring raw
  terminal mode is out, because hooks need the real terminal (from
  docs/design/0.2.0-rust-rewrite.md, high).

## Risks

- Starlark dialect drift: 0.2.0 runs `starlark-rust` where `v0.1.0` ran
  `go.starlark.net`, and runtime error text already differs (from
  docs/design/0.2.0-rust-rewrite.md, high). The `meowctl-stdlib` and `dotmeow`
  repositories are the corpus that catches a change in evaluation (from
  CLAUDE.md, high).
- Format drift: no `v0.1.0` binary runs beside the tests any more, so only the
  specification and the fixtures in `crates/meowctl-module/tests/fixtures/`
  catch a format change (from CLAUDE.md, high).
- Windows regressions: CI runs the tests on Linux only while the tree carries
  `cfg(windows)` code (from CLAUDE.md, high).

## Where it is going

Opening the Starlark API and the formats is a 0.3.0 decision, taken after parity
is proven (from CLAUDE.md and docs/design/0.2.0-rust-rewrite.md, high).
