---
id: TSK-0032
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-1023, REQ-1106, REQ-1264, REQ-1316, REQ-3035, REQ-3112, REQ-3113, REQ-3114, REQ-3115, REQ-3213, REQ-3304, REQ-3460, REQ-3461, REQ-3462, REQ-3463, REQ-3464, REQ-3465, REQ-3470, REQ-3520, REQ-3521, REQ-3522, REQ-3523, REQ-3524, REQ-3525, REQ-3526, REQ-3527, REQ-3528]
issue:
---

# The runtime hooks

This task records issue 36 of the 0.2.0 execution plan, which the plan marks
done, and closes 27 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a real configuration, when `meowctl hook shell` and
   `meowctl hook login` run, then each produces the same bytes as `v0.1.0` (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-common/src/paths.rs`,
   `crates/meowctl-config/tests/hookerr.rs`,
   `crates/meowctl-engine/tests/running.rs`,
   `crates/meowctl-release/tests/releases.rs`,
   `crates/meowctl-tui/tests/rendering.rs`, `tests/hook.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Make `meowctl hook shell` and `meowctl hook login` run the hooks of real
components (from docs/design/0.2.0-execution-plan.md, medium). `hook` was a
stub, and a shell spawn calls it, so a machine switched to `v0.2.0` lost its
PATH and every variable its components contribute (from
docs/design/0.2.0-execution-plan.md, high).

Two existing requirements were wrong. The one on `ctx.shell` named a
`shell.star` that does not exist, and the restricted surface for shell hooks was
never wired up; both were amended (from docs/design/0.2.0-execution-plan.md,
high).

`self-update` must not rename an unverified download over the running binary, as
`v0.1.0` does (from docs/design/0.2.0-execution-plan.md, high). The Windows home
directory must read `$USERPROFILE`, as `os.UserHomeDir` does (from
docs/design/0.2.0-execution-plan.md, high).

## Depends on

The plan blocks this issue on issue 34, the cutover, which closes no requirement
and so has no task; the epic lists it under Not covered (from
docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 36 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #82, "feat(cli): run the hooks a shell spawn runs", carried the
work, matched on the runtime-hook requirements it cites:
https://github.com/meowshed/meowctl/pull/82 (from
https://github.com/meowshed/meowctl/pull/82, high).

## Left alone

Verifying the download. The plan gives this issue the requirement that
`self-update` verifies, and issue 46 implemented it (from
docs/design/0.2.0-execution-plan.md, high).

The release checksum requirement. The plan lists it under this issue and under
issue 46; the epic gives it to issue 46, which built the release workflow that
publishes the file (inferred from docs/design/0.2.0-execution-plan.md, medium).
