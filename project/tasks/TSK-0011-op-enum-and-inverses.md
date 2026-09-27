---
id: TSK-0011
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-2001, REQ-2002, REQ-2003, REQ-2004, REQ-2005, REQ-2010, REQ-2011, REQ-2012, REQ-2013, REQ-2014, REQ-2015, REQ-2016, REQ-2017, REQ-2100, REQ-2101, REQ-2102, REQ-2103, REQ-2104, REQ-2105, REQ-2106, REQ-2107, REQ-2108, REQ-2109, REQ-2110, REQ-2111, REQ-2112, REQ-2113, REQ-2114, REQ-2115]
issue: 109
projected: 113bd985cc8e
---

# The Op enum and its inverses

This task records issue 12 of the 0.2.0 execution plan, which the plan marks
done, and closes 29 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a tree and any journalled operation, when the operation is applied and
   then its inverse, then the tree is restored, which the property test checks
   over every variant (from docs/design/0.2.0-execution-plan.md, high). Closed
   by: the tests that cite its requirements, in
   `crates/meowctl-ctx/tests/surface.rs`,
   `crates/meowctl-fs/tests/conformance.rs`,
   `crates/meowctl-ops/tests/inverses.rs`, `crates/meowctl-ops/tests/journal.rs`
   (from `rg REQ- crates tests`, medium).

## What to do

Add the `Op` enum: nine variants with nine inverses (from
docs/design/0.2.0-execution-plan.md, high). The issue is large on purpose. Split
by variant, each piece would have no behaviour, and the property test that
applying an operation then its inverse restores the state only means something
over the whole set (from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0009, issue 10 in the plan: every operation applies through `FileSystem`
  (inferred from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 12 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #59, "Make every effect a value with an inverse, and journal it",
carried the work, matched on "Issues 12 and 13" in its body:
https://github.com/meowshed/meowctl/pull/59 (from
https://github.com/meowshed/meowctl/pull/59, high).

## Left alone

The fetch behind `Op::Download`. The operation writes bytes somebody else
fetched, so it can be applied against an in-memory filesystem without a server
(from https://github.com/meowshed/meowctl/pull/59, high).
