---
id: TSK-0037
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3253, REQ-3254, REQ-3255, REQ-3256, REQ-3314, REQ-3315, REQ-3316, REQ-3317, REQ-3318]
issue: 135
projected: 1f88e3196fff
---

# The theme file

This task records issue 45 of the 0.2.0 execution plan, which the plan marks
done, and closes 9 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a `theme.toml`, and then a malformed one, when a command runs, then the
   theme reaches the drawn line, and the malformed one warns without stopping
   the command (from docs/design/0.2.0-execution-plan.md, high). Closed by: the
   tests that cite its requirements, in `crates/meowctl-tui/tests/rendering.rs`,
   `tests/theme.rs` (from `rg REQ- crates tests`, medium).

## What to do

Read `theme.toml` from the configuration directory, a table per role, over the
Catppuccin default (from docs/design/0.2.0-execution-plan.md, high). A role the
file leaves out keeps its default, a role it names is replaced whole, and a
misspelled role is refused (from docs/design/0.2.0-execution-plan.md, high).

Parsing takes the text, not a path, because `meowctl-tui` depends on nothing in
the workspace but `meowctl-common`. The binary reads the file before the sink is
built, because a sink is chosen once (from docs/design/0.2.0-execution-plan.md,
high). Absence is silent and the other two failures warn (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

The plan blocks this issue on issue 41, which amended requirements and closes
none, so it has no task (from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 45 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #92, "feat(tui): let a user point at their own palette", carried
the work, matched on the theme requirements it cites:
https://github.com/meowshed/meowctl/pull/92 (from
https://github.com/meowshed/meowctl/pull/92, high).

## Left alone

Nothing recorded.
