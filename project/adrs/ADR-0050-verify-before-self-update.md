---
id: ADR-0050
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-3470, REQ-3527, REQ-3528]
supersedes: []
---

# 0050. Verify a download before `self-update` installs it

## Decision

`self-update` checks the downloaded binary against the checksum the release
publishes in `checksums.sri`, and refuses to replace the running binary when
they disagree (from docs/spec/cli.md and
https://github.com/meowshed/meowctl/pull/93, high). REQ-3470, REQ-3527 and
REQ-3528 state it.

Once accepted, nothing that can answer for the download URL can replace a
user's `meowctl`. A release that doesn't publish `checksums.sri` is one
`self-update` refuses (from CLAUDE.md, high).

## Why

`v0.1.0` fetches a release asset over HTTPS and renames it over
`os.Executable()` with no check of any kind, and reproducing that would ship
the weakness on purpose in a binary that already verifies every module it
downloads (from docs/spec/cli.md, high). This is the one place the parity
constraint doesn't reach, and ADR-0002 left it unsettled (from
docs/spec/cli.md, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Reproduce `v0.1.0`: install what was downloaded | Parity, and a release needs no checksum file | Anything that answers for the URL replaces the binary (from docs/spec/cli.md, high) |

## What it costs

Every release has to publish `checksums.sri`, and
`.github/workflows/release.yml` is what produces it; a release without it can't
be installed by `self-update` (from CLAUDE.md and REQ-3471, high).

## What would reverse it

- The trade-off row names no condition, and none is foreseeable while
  `self-update` replaces a binary it downloaded (from
  docs/design/0.2.0-requirement-tradeoffs.md, high).

## Consequences

- The release workflow is part of the product, not dressing (from CLAUDE.md,
  high).

## How I will know it was realised

1. `crates/meowctl-release/tests/releases.rs` refuses bytes whose hash doesn't
   match, naming both hashes, and holds REQ-3470, REQ-3527 and REQ-3528 (from
   crates/meowctl-release/tests/releases.rs:76-89, high).

## What this does not settle

- Whether the checksum file should itself be signed.
