---
id: ADR-0045
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1223]
supersedes: []
---

# 0045. Delete the compatibility corpus with the Go tree, and keep three fixtures

## Decision

At the cutover the compatibility corpus goes with the Go tree, along with the
five requirements that described it and `meowctl-xtask`; a `deps.lock` `v0.1.0`
wrote and the two tarballs it hashed survive in
`crates/meowctl-module/tests/fixtures/` (from
https://github.com/meowshed/meowctl/pull/80 and
docs/design/0.2.0-execution-plan.md issue 34, high). The fixtures are the lock
format itself, which REQ-1223 freezes (from docs/design/0.2.0-execution-plan.md
issue 34, high).

Once accepted, nothing regenerates an oracle; a format change is caught by the
specification and the fixtures (from CLAUDE.md, high).

## Why

Keeping the oracle past the cutover makes it a brake: every deliberate
improvement would have to be argued past a corpus that says the old behaviour is
correct, and producing the corpus means keeping the Go tree buildable (from
https://github.com/meowshed/meowctl/pull/80, high). Parity is the constraint of
the rewrite, not of the tool (from docs/design/0.2.0-execution-plan.md issue 34,
high). Issue 33 had passed the corpus, and both binaries were run against the
same live configuration first (from https://github.com/meowshed/meowctl/pull/80,
high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Keep the oracle after the cutover | Keeps a diff against `v0.1.0` for every later change | It would brake every deliberate improvement and keep the Go tree buildable (from https://github.com/meowshed/meowctl/pull/80, high) |

## What it costs

The requirement count dropped from 312 to 306, and parity after the cutover
rests on the specification and three fixtures rather than on a diff against a
binary (from https://github.com/meowshed/meowctl/pull/80 and CLAUDE.md, high).

## What would reverse it

- The source names no reversal condition (from
  https://github.com/meowshed/meowctl/pull/80, high).

## Consequences

- Trade-off rows whose reversal read "the Go tree is deleted" had that condition
  met, which makes each a decision somebody can take; none has been taken (from
  https://github.com/meowshed/meowctl/pull/80 and
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## How I will know it was realised

1. `tests/compat` and `crates/meowctl-xtask` don't exist, and
   `crates/meowctl-module/tests/fixtures/` holds `deps.lock.synced` and two
   tarballs (from `ls tests crates crates/meowctl-module/tests/fixtures`, high).
2. `crates/meowctl-module/tests/syncing.rs` holds REQ-1223 against those
   fixtures (from crates/meowctl-module/tests/syncing.rs, high).

## What this does not settle

- The source recorded no alternative beyond keeping the oracle (from
  https://github.com/meowshed/meowctl/pull/80, high).
