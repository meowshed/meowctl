---
id: ADR-0048
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3570]
supersedes: []
---

# 0048. Run the test matrix on Linux only, while releases build for four targets

## Decision

CI runs the test matrix on `ubuntu-latest` alone, and the release workflow
still builds all four targets REQ-3570 names (from .github/workflows/ci.yml:70
and .github/workflows/release.yml:23-30, high). Before a change touches code
behind a `cfg`, its author runs `mise run check-windows` (from CLAUDE.md,
high).

Once accepted, a pull request waits on one operating system. What doesn't work
yet: a macOS or Windows regression is caught only by `mise run check-windows`
or by a release build (from https://github.com/meowshed/meowctl/pull/91,
high).

## Why

The last two Windows failures were an unused import and a path literal that is
absolute on one platform and not the other, so the matrix cost a run per
platform to catch mistakes a cross-platform check finds (from
https://github.com/meowshed/meowctl/pull/91 and .github/workflows/ci.yml:58-62,
high). The pull request calls the change reversible and says it should be
reversed (from https://github.com/meowshed/meowctl/pull/91, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Keep macOS and Windows on the test matrix | Every shipped platform is tested before merge | Each run paid for three platforms to catch failures a check catches (from https://github.com/meowshed/meowctl/pull/91, high) |
| Ship Linux alone | What ships is what is tested | meowctl manages dotfiles on macOS too, and `self-update` expects an asset per platform (from .github/workflows/release.yml, medium) |

## What it costs

Three of the four targets REQ-3570 names ship untested: a `cfg(windows)` or
macOS regression reaches `main` unless its author ran `mise run check-windows`
(from https://github.com/meowshed/meowctl/pull/91 and
.github/workflows/ci.yml:58-62, high).

## What would reverse it

- macOS and Windows go back on the matrix, which the pull request and the
  workflow both say is intended (from
  https://github.com/meowshed/meowctl/pull/91 and .github/workflows/ci.yml:58,
  high).

## Consequences

- `mise run check-windows` is the step left before touching code behind a
  `cfg` (from CLAUDE.md, high).

## How I will know it was realised

1. `.github/workflows/ci.yml` sets `os: [ubuntu-latest]`, and
   `.github/workflows/release.yml` builds the four targets (from those files,
   high).

## What this does not settle

- When macOS and Windows return to the matrix; the pull request names no date.
