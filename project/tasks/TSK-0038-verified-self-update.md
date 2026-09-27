---
id: TSK-0038
artifact: task
status: approved
revised: 2026-09-27
epic: EPC-0001
closes: [REQ-3471, REQ-3472, REQ-3473, REQ-3474, REQ-3475, REQ-3476, REQ-3529, REQ-3530, REQ-3531, REQ-3532]
issue: 136
projected: 57f72694de7f
---

# Updating the binary, verified

This task records issue 46 of the 0.2.0 execution plan, which the plan marks
done, and closes 10 requirements (from docs/design/0.2.0-execution-plan.md,
high).

## Acceptance criteria

1. Given a release with no checksums, and one whose checksum does not match,
   when `self-update` runs, then each is refused, the mismatch naming both
   hashes, and the replacement arrives whole or not at all (from
   docs/design/0.2.0-execution-plan.md, high). Closed by: the tests that cite
   its requirements, in `crates/meowctl-cli/src/update.rs`,
   `crates/meowctl-cli/tests/surface.rs`,
   `crates/meowctl-release/tests/releases.rs`, `tests/errors.rs` (from
   `rg REQ- crates tests`, medium).

## What to do

Make `self-update` verify what it downloads. The release half: `release.yml`
builds four targets, names each `meowctl-<target triple>` and writes
`checksums.sri`, one SHA-384 line per asset (from
docs/design/0.2.0-execution-plan.md, high). A release without that file is
refused (from docs/design/0.2.0-execution-plan.md, high).

The binary half: `meowctl-release` takes bytes somebody else fetched and decides
which asset belongs to this platform, which hash the release published for it,
and whether the bytes are it, with no network and no filesystem (from
docs/design/0.2.0-execution-plan.md, high). `meowctl-cli` keeps the fetching and
the replacing (from docs/design/0.2.0-execution-plan.md, high).

The replacement is written beside the binary, made executable, then renamed over
it. `MEOWCTL_RELEASES` points the update elsewhere, for a fork and for the tests
(from docs/design/0.2.0-execution-plan.md, high).

## Depends on

The plan blocks this issue on issue 41, which amended requirements and closes
none, so it has no task (from docs/design/0.2.0-execution-plan.md, high).

## Evidence

The plan marks issue 46 done (from docs/design/0.2.0-execution-plan.md, high).

Pull request #93, "feat(cli): update the binary, and verify it first", carried
the work, matched on the self-update requirements it cites:
https://github.com/meowshed/meowctl/pull/93 (from
https://github.com/meowshed/meowctl/pull/93, high).

## Left alone

Nothing recorded.
