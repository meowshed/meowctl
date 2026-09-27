---
id: ADR-0031
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3016]
supersedes: []
---

# 0031. Fail when a component filter matches nothing

## Decision

A filter naming components restricts the run to those and their dependencies,
and a name matching nothing is an error rather than an empty run (from
docs/spec/engine.md, high). REQ-3016 states it (from
docs/design/0.2.0-decisions.md §3.13, high).

Once accepted, `meowctl apply nvim` for a component spelled `neovim` fails.

## Why

Otherwise `meowctl apply nvim` reports success and does nothing, which is the
worse outcome by a wide margin; a script that needs the other behaviour can test
first (from docs/design/0.2.0-decisions.md §3.13, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Run nothing | A script can filter on a component that's absent on some machines | A misspelt name reports success and does nothing (from docs/design/0.2.0-decisions.md §3.13, high) |

## What it costs

Failing is unhelpful in a script that filters on a component that may
legitimately be absent on some machines (from docs/design/0.2.0-decisions.md
§3.13, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.13, high).

## Consequences

- A filter keeps what the named components depend on, so the plan never promises
  a tool without its package manager (from
  https://github.com/meowshed/meowctl/pull/73, high).

## How I will know it was realised

1. `crates/meowctl-engine/tests/plan.rs` and
   `crates/meowctl-engine/tests/graph.rs` hold REQ-3016 (from those files,
   high).

## What this does not settle

- The source recorded no alternative beyond running nothing (from
  docs/design/0.2.0-decisions.md §3.13, high).
