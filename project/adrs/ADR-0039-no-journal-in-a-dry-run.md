---
id: ADR-0039
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2026]
supersedes: []
---

# 0039. Journal nothing during a dry run

## Decision

No `Op` is journalled during a dry run, and no journal file is created (from
docs/spec/ops.md, high). REQ-2026 states it (from docs/design/0.2.0-decisions.md
§3.23, high).

Once accepted, a dry run leaves nothing a later run would replay.

## Why

A journal of operations that didn't happen would replay into damage (from
docs/design/0.2.0-decisions.md §3.23, high). `v0.1.0` has no journal in a dry
run to forbid (from docs/design/0.2.0-decisions.md §3.23, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Journal the operations a dry run records | One journal path for both kinds of run | It would replay operations that never happened (from docs/design/0.2.0-decisions.md §3.23, high) |

## What it costs

The journal is the one effect outside `FileSystem`: it reads and writes through
`std::fs`, so `DryRunFs` can't make it a no-op (from
crates/meowctl-ops/src/journal.rs:87-99, high). Its absence in a dry run rests
on `meowctl-cli` returning before `Journal::open`, and only a test that drives
the binary proves it (from https://github.com/meowshed/meowctl/pull/85 and
crates/meowctl-cli/src/run/commands.rs:449-463, medium). A dry run also can't
predict a real run failing to open the journal, which ADR-0023 asks of a
failure a dry run can see (inferred, low).

## What would reverse it

- The journal moves behind `FileSystem`, where `DryRunFs` would make it a no-op
  by construction and the control flow in `meowctl-cli` would no longer carry
  the guarantee (from crates/meowctl-ops/src/journal.rs:87-99, medium).

## Consequences

- A dry run writes no sentinel either (from
  https://github.com/meowshed/meowctl/pull/85, high).

## How I will know it was realised

1. `tests/dry_run.rs` drives the binary and checks no journal exists, holding
   REQ-2026; the check lives there because the journal is never constructed
   (from https://github.com/meowshed/meowctl/pull/85 and tests/dry_run.rs,
   high).

## What this does not settle

- The source recorded no alternative beyond a journal of unperformed operations
  (from docs/design/0.2.0-decisions.md §3.23, high).
