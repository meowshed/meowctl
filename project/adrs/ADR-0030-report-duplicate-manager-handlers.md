---
id: ADR-0030
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-2604]
supersedes: []
---

# 0030. Report two handlers that claim one package manager

## Decision

Two components exporting the same `pm_name` are reported rather than silently
resolved (from docs/spec/pm.md, high). REQ-2604 states it, and the change is to
be said in the release notes (from docs/design/0.2.0-decisions.md §3.12, high).

Once accepted, a configuration's meaning no longer depends on evaluation order.
The `CHANGELOG.md` Unreleased section doesn't mention it yet (from CHANGELOG.md,
medium).

## Why

A configuration whose meaning depends on graph order isn't working, it's
coincidentally correct (from docs/design/0.2.0-decisions.md §3.12, high).
`v0.1.0`'s `Register` overwrites, so two components claiming `brew` produce
whichever evaluated last (from https://github.com/meowshed/meowctl/pull/69,
high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Overwrite, as `v0.1.0` does | Keeps working a configuration that shadows a stdlib handler | Which handler wins depends on graph order (from docs/design/0.2.0-decisions.md §3.12, high) |

## What it costs

Reporting can break a configuration that works today by accident, where a user
shadows a stdlib handler with their own and relies on evaluation order (from
docs/design/0.2.0-decisions.md §3.12, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.12, high).

## Consequences

- Release notes are to say so (from docs/design/0.2.0-decisions.md §3.12, high).

## How I will know it was realised

1. `crates/meowctl-pm/tests/registry.rs` and
   `crates/meowctl-engine/tests/graph.rs` hold REQ-2604 (from those files,
   high).
2. `rg -n 'pm_name|duplicate|claim' CHANGELOG.md` finds no release note for the
   change (from CHANGELOG.md, medium).

## What this does not settle

- The source recorded no alternative beyond overwriting (from
  docs/design/0.2.0-decisions.md §3.12, high).
