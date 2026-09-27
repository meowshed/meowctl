---
id: ADR-0002
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1223, REQ-2201, REQ-3401]
supersedes: []
---

# 0002. Keep the Starlark API, the command surface and every format compatible with v0.1.0

## Decision

`v0.2.0` ships no new user-facing features: the `init.star`, `deps.mod`,
`deps.lock`, `pkgs.lock` and `state.toml` formats stay byte-compatible, the
Starlark API is unchanged down to the builtin signatures, the command surface
stays the same, and an existing dotfiles repository applies identically under
both binaries (from docs/design/0.2.0-rust-rewrite.md §1, high). REQ-1223,
REQ-2201 and REQ-3401 state the three halves of that as obligations (from
docs/spec/config.md, docs/spec/starlark.md and docs/spec/cli.md, high). New
commands, Starlark API changes, config format changes, parallel component
execution, a daemon and wider Windows support are each a 0.3.0 conversation,
after parity is proven (from docs/design/0.2.0-rust-rewrite.md §8, high).

Once accepted, `v0.1.0` is an oracle a diff can check the rewrite against.
Terminal output is exempt; ADR-0003 records that carve-out.

## Why

Parity is what makes the rewrite verifiable, because `v0.1.0` becomes an oracle
that can be diffed against; give it up and the only way to know the rewrite is
correct is to read it (from docs/design/0.2.0-decisions.md §1, high). The same
constraint also holds scope still: the §2 defect list is the whole licence to
change design (from docs/design/0.2.0-rust-rewrite.md §5, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| A rewrite that also improves the surface | Fixes surface problems in the same release as the architecture | Nothing could then check the rewrite except reading it (from docs/design/0.2.0-decisions.md §1, high) |

## What it costs

Each requirement that exists only to match `v0.1.0` is a place the project pays
for a constraint it chose, and a row whose reversal reads "parity is abandoned"
has no independent justification (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Improvements to the surface
wait for 0.3.0 (from docs/design/0.2.0-rust-rewrite.md §8, high).

## What would reverse it

- 0.3.0 opens the API, which needs `/amend-spec` rather than a pull request that
  widens a signature (from CLAUDE.md, high).
- The Go tree being deleted met the reversal condition of several trade-off
  rows, but none of those decisions has been taken (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- The compatibility corpus recorded `v0.1.0`'s behaviour and checked `v0.2.0`
  against it until the cutover (from docs/design/0.2.0-rust-rewrite.md §6 and
  https://github.com/meowshed/meowctl/pull/78, high).
- Most requirements are transcribed or derived rather than decided: 42 of 337
  are decided. The trade-off table marks 42 rows, and it and
  `docs/design/0.2.0-decisions.md` both say 40 (from a count of the rows marked
  decided in docs/design/0.2.0-requirement-tradeoffs.md, high).
- After the cutover, a format change is caught by the specification and by the
  fixtures in `crates/meowctl-module/tests/fixtures/`, not by a diff against a
  binary (from CLAUDE.md, high).

## How I will know it was realised

1. `cargo xtask compat check` reported "no differences against v0.1.0" on 28
   cases before the Go tree was deleted (from
   https://github.com/meowshed/meowctl/pull/78, high).
2. A lock `v0.2.0` writes is byte-identical to the `v0.1.0` fixture outside
   `meta`: `crates/meowctl-config/tests/parity.rs` and
   `crates/meowctl-module/tests/syncing.rs` hold REQ-1223 (from those files,
   high).
3. The command set matches `newRootCmd`: `crates/meowctl-cli/tests/surface.rs`
   holds REQ-3401 (from crates/meowctl-cli/tests/surface.rs, high).

## What this does not settle

- Terminal output, which ADR-0003 exempts.
- `self-update`, where `v0.2.0` deliberately does not reproduce the unverified
  download; ADR-0050 records that.
- The source recorded no alternative beyond improving the surface in the same
  release (from docs/design/0.2.0-decisions.md §1, high).
