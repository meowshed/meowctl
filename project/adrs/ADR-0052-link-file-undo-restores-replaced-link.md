---
id: ADR-0052
artifact: adr
status: done
revised: 2026-10-31
addresses: [REQ-2033]
supersedes: []
---

# 0052. Restore the symlink a `link_file` replaced when it is undone

## Decision

When `link_file` replaces an existing symlink, its journal record carries the
prior target in a `prior_target` field beside `"was_backed_up": false`, and
undoing it re-creates that symlink. A `link_file` that created a new link, or
that moved a regular file aside, undoes as before.

Once accepted, a failed run leaves a re-pointed config link where it was before
the run. A `v0.1.0` binary replaying the same record ignores `prior_target`
and removes the link, as it always did.

## Why

`v0.1.0`'s `prepareLinkFileDst` replaces an existing symlink without a backup
and records only `dst` and `was_backed_up`, so its undo removes the link. Every
module upgrade re-points every config link at the new cache directory, so any
failure in a later component turns into deleted config files: on 2026-10-31 one
`ctx.prompt` in a non-interactive run removed 24 links, `~/.ssh/config` among
them (BUG-0045). `Op::inverse` already computes the prior symlink as REQ-1506
and REQ-2003 require; only the journal drops it. Keeping `v0.1.0`'s undo here
would ship the data loss on purpose, the same reasoning ADR-0050 applies to
`self-update`.

## Alternatives

| Option | Better at | Why it lost |
| --- | --- | --- |
| Keep `v0.1.0`'s undo, which removes the link | The record and its replay match `v0.1.0` exactly | A failed run deletes the user's config files |
| Journal nothing for a link that replaced a link | Undo leaves the new link, which still works | The new link may point at a cache directory a later step removes, and the run is no longer undone |

## What it costs

The `link_file` record gains a field `v0.1.0` doesn't write, which REQ-2190
allows because `v0.1.0` ignores an unknown field, as it already does for
`mkdir`'s `until`. A `v0.1.0` replay of such a record still removes the link.

## What would reverse it

- Parity with `v0.1.0`'s replay becomes stricter than "replays without
  failing", so that the same record must mean the same undo in both binaries.

## Consequences

- REQ-2033 supersedes REQ-2015, and SPC-2000 states it in its place.

## How I will know it was realised

1. `crates/meowctl-ops/tests/journal.rs` writes a `link_file` record over a
   replaced symlink, reads it back and gets an inverse that re-creates the
   prior link, and holds REQ-2033.
