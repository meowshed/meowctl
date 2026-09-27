---
id: ADR-0004
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1414, REQ-2004, REQ-2814, REQ-3002, REQ-3410, REQ-3411, REQ-1411, REQ-1504, REQ-1505, REQ-3505, REQ-3506]
supersedes: []
---

# 0004. Put effects behind traits constructed once at the binary

## Decision

`FileSystem` and `Executor` are constructed once in `main` from the flags and
passed down; `--dry-run` selects `DryRunFs`, and no layer below `meowctl-cli`
reads a global or checks a dry-run flag while it does work (from
docs/design/0.2.0-rust-rewrite.md §3, high). A dry run is a different
implementation, not a branch (from docs/design/0.2.0-decisions.md §2, high).
REQ-3410, REQ-3411, REQ-2814 and REQ-1414 are where this is enforceable in
review (from docs/design/0.2.0-decisions.md §3.16 and docs/spec/cli.md, high).

The engine's `dry_run: bool` is the allowed selection of an executor, not a
breach: it is read once per phase to construct an implementation and once to
fill `ctx.dry_run`, and no code asks whether this is a dry run while it does
work (from crates/meowctl-engine/src/runner.rs:382 and :567, high). REQ-3505
names those two uses and REQ-3506 forbids every other, and REQ-1411, REQ-1504
and REQ-1505 are the `DryRunFs` half of the rule, which rejected skipping
effectful calls at the call site (from
docs/design/0.2.0-requirement-tradeoffs.md:67, high).

Once accepted, the engine is unit-testable against `MemFs`, and a dry run can't
write. What doesn't work yet: the binary never sets the engine's flag, so the
per-phase executor runs only in engine tests; BUG-0020 records that (from
crates/meowctl-cli/src/run/commands.rs:481 and :598, high).

## Why

A boolean has to reach `ctx` whatever carries it, because `ctx.dry_run` is part
of the frozen `v0.1.0` surface; the literal ban on passing the flag down as a
boolean couldn't hold, and the property worth holding is that a dry run is a
different implementation, not a different code path (from docs/spec/ctx.md:30
and CLAUDE.md, high).

`fix(apply): dry-run claimed work that the runner skips` is the bug this
prevents, and it recurs: the guarantee "a dry run writes nothing" held only as
long as every author remembered a branch (from docs/design/0.2.0-decisions.md
§2, high). `v0.1.0` checks `c.caps.DryRun` in a dozen places and missed one
(from https://github.com/meowshed/meowctl/pull/57, high). The change is also
what makes the engine unit-testable, and so what makes REQ-2004 and REQ-3002
checkable at all (from docs/design/0.2.0-rust-rewrite.md §3 and
docs/design/0.2.0-decisions.md §2, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Keep the `v0.1.0` shape: a `dry_run` flag in a context struct, checked where it matters | No parameter threaded through every signature, and a direct call instead of a virtual one | The guarantee depends on every author remembering a branch, and one was missed (from docs/design/0.2.0-decisions.md §2, high) |

## What it costs

Every function that touches the filesystem grows a parameter, and a trait object
has a virtual call where `v0.1.0` had a direct one (from
docs/design/0.2.0-decisions.md §2, high). The virtual call isn't measurable in a
tool that waits on subprocesses and the network; the parameter is real and shows
up in signatures throughout `meowctl-ops` and `meowctl-engine` (from
docs/design/0.2.0-decisions.md §2, high).

## What would reverse it

- A measurement shows the indirection costs something on `meowctl hook shell`,
  the one latency-sensitive path (from docs/design/0.2.0-decisions.md §2, high).

## Consequences

- `meowctl-fs` holds `RealFs`, `DryRunFs` and `MemFs`, and `meowctl-exec` holds
  `RealExecutor`, `DryRunExecutor` and `ScriptedExecutor` (from
  https://github.com/meowshed/meowctl/pull/57 and
  https://github.com/meowshed/meowctl/pull/58, high).
- A new `if dry_run` anywhere in the tree is a review finding (from CLAUDE.md,
  high).
- Runner tests run real Starlark hooks against a `MemFs` and a scripted
  executor, with no machine to run on (from
  https://github.com/meowshed/meowctl/pull/74, high).

## How I will know it was realised

1. `crates/meowctl-fs/tests/conformance.rs` runs its cases against all three
   filesystem implementations and holds REQ-1414 (from
   https://github.com/meowshed/meowctl/pull/57 and
   crates/meowctl-fs/tests/conformance.rs, high).
2. `tests/dry_run.rs` drives the binary under `--dry-run` and holds REQ-3410 and
   REQ-3411 (from tests/dry_run.rs, high).
3. `rg dry_run crates --glob '!crates/meowctl-cli/**'` finds the flag only in
   `meowctl-engine`'s settings, `effects_for` and the `ctx.dry_run` property;
   confidence that nothing else branches is medium, because the check is a
   search rather than a lint (from crates/meowctl-engine/src/runner.rs and
   crates/meowctl-ctx/src/value.rs, medium).

## What this does not settle

- Whether a dry run should run the read-only phases at all; BUG-0020 holds
  that question.
- The source recorded no alternative beyond the `v0.1.0` shape (from
  docs/design/0.2.0-decisions.md §2, high).
