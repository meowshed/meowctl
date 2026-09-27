---
id: TSK-0030
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3401, REQ-3402, REQ-3404, REQ-3405, REQ-3406, REQ-3410, REQ-3411, REQ-3412, REQ-3413, REQ-3414, REQ-3420, REQ-3421, REQ-3422, REQ-3430, REQ-3431, REQ-3432, REQ-3433, REQ-3450, REQ-3451, REQ-3452, REQ-3501, REQ-3502, REQ-3503, REQ-3504, REQ-3505, REQ-3506, REQ-3507, REQ-3510, REQ-3511, REQ-3512, REQ-3513, REQ-3514, REQ-3515, REQ-3516, REQ-3517, REQ-3518]
issue: 128
projected: 8c4dbc12e21b
---

# The command tree and the commands that read

This task records issue 32 of the 0.2.0 execution plan, which the plan marks
done, and closes 36 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given the recorded `v0.1.0` corpus, when the Rust binary runs each command in
   it, then each produces the recorded files and exit code (from
   https://github.com/meowshed/meowctl/pull/76, medium). Closed by: the tests
   that cite its requirements, in `crates/meowctl-cli/tests/scaffold.rs`,
   `crates/meowctl-cli/tests/surface.rs`,
   `crates/meowctl-config/tests/parity.rs`, `tests/dry_run.rs`,
   `tests/errors.rs`, `tests/interrupt.rs` (from `rg REQ- crates tests`,
   medium).

## What to do

Build the whole command tree, the construction of effects, the exit codes, and
every command that reads or applies: `apply`, `upgrade`, `verify`, `status`,
`doctor`, `check`, `hook`, `shell`, `dep list`, `dep sync`, `version` and
completions (from docs/design/0.2.0-execution-plan.md, high). The issue is large
on purpose. These are the commands the corpus runs, so this issue makes the
corpus check possible (from docs/design/0.2.0-execution-plan.md, high).

## Depends on

- TSK-0028, issue 30 in the plan: the commands drive the runner (inferred from
  docs/design/0.2.0-execution-plan.md, high).
- TSK-0023, issue 25 in the plan: the commands write through the plain and JSON
  sinks (inferred from docs/design/0.2.0-execution-plan.md, medium).

## Evidence

The plan marks issue 32 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #76, "feat(cli): give the rewrite a binary, and pass the corpus",
carried the work, matched on "Issue 32" in its body:
https://github.com/meowshed/meowctl/pull/76 (from
https://github.com/meowshed/meowctl/pull/76, high).

## Left alone

The commands that write a configuration, split into issue 35 (from
docs/design/0.2.0-execution-plan.md, high).
