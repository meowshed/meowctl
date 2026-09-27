---
id: ADR-0049
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1104, REQ-1105]
supersedes: []
---

# 0049. Refuse relative paths and `~user` in path resolution

## Decision

Path resolution rejects a path that isn't absolute once expanded, including
`~user`, and rejects a path that escapes the directory it was resolved
against, where `v0.1.0` accepts both (from docs/spec/common.md, high).
REQ-1104 and REQ-1105 state it.

Once accepted, a hook's path means the same thing whatever the process working
directory is. A configuration that passed `v0.1.0` with a relative path fails
under `v0.2.0`.

## Why

A hook has no defined working directory, so a relative path resolves somewhere
its author can't predict, and `expandPath` passes `~user` through, which
creates a directory literally named `~user` (from docs/spec/common.md and
docs/design/0.2.0-requirement-tradeoffs.md:43, high). The row is marked decided
and deliberately breaks `v0.1.0` behaviour, which ADR-0002 treats as a
decision, and `docs/design/0.2.0-decisions.md` doesn't argue it (from
docs/design/0.2.0-requirement-tradeoffs.md, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Accept relative paths and `~user`, as `v0.1.0` does | Every configuration `v0.1.0` applied still applies | A relative path resolves somewhere nobody predicts, and `~user` becomes a directory of that name (from docs/design/0.2.0-requirement-tradeoffs.md:43, high) |

## What it costs

A configuration that worked under `v0.1.0` with a relative path or `~user`
stops working, which the parity principle otherwise forbids (from CLAUDE.md
and REQ-1104, high).

## What would reverse it

- A hook gains a defined working directory (from
  docs/design/0.2.0-requirement-tradeoffs.md:43, high).

## Consequences

- The changelog has to name the break, because it tells users a configuration
  `v0.1.0` applied applies here (from CHANGELOG.md, high).

## How I will know it was realised

1. The unit tests in `crates/meowctl-common/src/paths.rs` reject a relative
   path and hold REQ-1022, REQ-1104 and REQ-1105 (from
   crates/meowctl-common/src/paths.rs:293-304, high).

## What this does not settle

- Whether `~user` should expand to that user's home in 0.3.0.
