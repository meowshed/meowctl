---
id: TSK-0007
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1250, REQ-1251, REQ-1252, REQ-1253, REQ-1314]
issue: 105
projected: 54c5a865321a
---

# The syntax-aware Starlark editor

This task records issue 8 of the 0.2.0 execution plan, which the plan marks
done, and closes 5 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a configuration file with comments and hand formatting, when the editor
   adds or removes a component, then every byte outside the edit is unchanged,
   and the evaluator accepts the result (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-config/tests/editing.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Edit a Starlark configuration file by parsing it. The edit is made to the text,
at the position the parse reports, so nothing is re-rendered and nothing can be
lost (from docs/design/0.2.0-execution-plan.md, high). A property test proves
the untouched bytes stay untouched (from docs/design/0.2.0-execution-plan.md,
high).

`meowctl-config` gains a dependency on the `starlark` crate for its parser, an
external dependency of the same shape as `toml` and not an edge to
`meowctl-starlark` (from docs/design/0.2.0-execution-plan.md, high).

The dialect is then named in two crates, so a test runs one file through both. A
file the editor accepts and the evaluator rejects would let `meowctl add`
succeed on a configuration that then fails to apply (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0006, issue 7 in the plan: the editor edits the files whose layout issue 7
  fixes (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 8 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #63, "Edit a Starlark file by parsing it, not by matching text",
carried the work, matched on "Issues 7 and 8" in its body:
https://github.com/meowshed/meowctl/pull/63 (from
https://github.com/meowshed/meowctl/pull/63, high).

## Left alone

A printer. The first sizing assumed formatting preservation needed one, and it
does not, which is why the issue came in at medium (from
docs/design/0.2.0-execution-plan.md, high).
