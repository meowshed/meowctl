---
id: ADR-0027
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1250, REQ-1251, REQ-1252]
supersedes: []
---

# 0027. Edit Starlark configuration files by parsing them, not by regular expression

## Decision

Config edits are made through a syntax-aware editor over the parsed Starlark,
preserving formatting and comments, in place of the regular expressions in
`internal/rewrite/` (from docs/design/0.2.0-rust-rewrite.md §2 defect #5 and
docs/design/0.2.0-decisions.md §3.9, high). REQ-1250, REQ-1251 and REQ-1252
state it (from docs/design/0.2.0-decisions.md §3.9, high).

Once accepted, a `dep()` is found in either keyword order, and an edit that
finds nothing is reported rather than written as a no-op (from
docs/design/0.2.0-decisions.md §3.9, high). The edit is made to the text at the
position the parse reports, so nothing is re-rendered (from
https://github.com/meowshed/meowctl/pull/63, high).

## Why

The regular expression carries a documented limitation, that `dep()` must be
written `name` before `version`, and a user who hand-edits `deps.mod` into the
other order gets "not found" for a declaration that is plainly there (from
docs/design/0.2.0-decisions.md §3.9, high). Parsing also tells a commented-out
declaration and a name inside a string from a declaration, which `v0.1.0` gets
wrong or can't see (from https://github.com/meowshed/meowctl/pull/63, high).
Reporting a missed edit is what lets `AppendComponent` stop appending duplicates
(from docs/design/0.2.0-decisions.md §3.9, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Port the regular expressions from `internal/rewrite/` | Forty lines that work on files the tool wrote | A hand-edited file in the other keyword order reports "not found" (from docs/design/0.2.0-decisions.md §3.9, high) |

## What it costs

docs/design/0.2.0-decisions.md §3.9 priced parsing as an AST, a printer that
preserves formatting, and the property tests to prove it (from
docs/design/0.2.0-decisions.md §3.9, high). The printer turned out unnecessary:
formatting is preserved by editing the text at parsed positions, which is why
issue 8 came in smaller than planned (from
https://github.com/meowshed/meowctl/pull/63, high).

## What would reverse it

- The source names no reversal condition; the trade-off rows for REQ-1250 read
  "Never" (from docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- `meowctl-config` takes `starlark_syntax` as an external dependency; ADR-0041
  records that (from https://github.com/meowshed/meowctl/pull/63, high).
- Every editing test asserts on the whole file rather than on the changed line
  (from https://github.com/meowshed/meowctl/pull/63, high).

## How I will know it was realised

1. `crates/meowctl-config/tests/editing.rs` holds REQ-1250, REQ-1251 and
   REQ-1252 (from crates/meowctl-config/tests/editing.rs, high).

## What this does not settle

- The source recorded no alternative beyond porting the regular expressions
  (from docs/design/0.2.0-decisions.md §3.9, high).
