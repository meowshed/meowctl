---
id: ADR-0021
artifact: adr
status: approved
revised: 2026-09-27
addresses: [REQ-1404]
supersedes: []
---

# 0021. Create directories with mode 0700

## Decision

A created directory gets mode `0o700` and a created file `0o600`, except where a
caller passes a mode (from docs/spec/fs.md, high). REQ-1404 states it (from
docs/design/0.2.0-decisions.md §3.2, high).

Once accepted, every directory meowctl creates is private to the user.

## Why

`v0.1.0` creates the same config directory `0755` from `internal/lock/write.go`
and `0700` from `internal/state/state.go`, and reproducing an inconsistency is
transcription of a bug, not parity (from docs/design/0.2.0-decisions.md §3.2,
high). The directory holds whatever a user's components put in it, and the
narrow default is the safe one (from docs/design/0.2.0-decisions.md §3.2, high).

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Reproduce `v0.1.0`'s two modes | Byte-for-byte behaviour with each `v0.1.0` writer | It transcribes a bug (from docs/design/0.2.0-decisions.md §3.2, high) |

## What it costs

`0700` can break a setup where another user reads the config directory, which is
unusual for dotfiles (from docs/design/0.2.0-decisions.md §3.2, high).

## What would reverse it

- The source names no reversal condition (from docs/design/0.2.0-decisions.md
  §3.2, high).

## Consequences

- `meowctl-fs` exposes setting the executable bit as the one exception, for
  scripts a module ships (from docs/spec/fs.md, high).

## How I will know it was realised

1. `crates/meowctl-fs/src/real.rs` sets `DIR_MODE` to `0o700`, and
   `crates/meowctl-fs/tests/conformance.rs` holds REQ-1404 (from those files,
   high).

## What this does not settle

- The source recorded no option beyond reproducing `v0.1.0` or picking one mode
  (from docs/design/0.2.0-decisions.md §3.2, high).
