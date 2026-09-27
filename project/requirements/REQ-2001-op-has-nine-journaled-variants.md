---
id: REQ-2001
artifact: requirement
topic: ops
class: functional
status: approved
revised: 2026-09-27
elaborates:
verification: behavioural
---

# REQ-2001

`Op` MUST have exactly these nine journalled variants: `WriteFile`,
`AppendFile`, `CopyFile`, `Symlink`, `LinkFile`, `Mkdir`, `Download`,
`DefaultsWrite`, and `PlistSet`.

The other four, `Remove`, `RemoveDir`, `RestoreBackup` and `Nothing`, are what
an inverse is: undoing a `Symlink` that created a link is removing it, and
undoing a `Mkdir` of a directory that already existed is doing nothing (from
docs/spec/ops.md, high). Without them `inverse()` could not return an `Op` (from
docs/spec/ops.md, high). `is_journaled` is the line between the two groups, and
REQ-2020 is what makes it matter: a journal written by one binary has to replay
under the other, and `v0.1.0` knows only the nine (from docs/spec/ops.md, high).
The alternatives were an open trait any crate can implement, or exactly nine
variants with no inverse-only ones (from
docs/design/0.2.0-requirement-tradeoffs.md, high). An open set cannot be
replayed by a binary that does not know the variant, and exactly nine leaves
`inverse()` nothing to return, which is the whole of operations being data (from
docs/design/0.2.0-requirement-tradeoffs.md, high). Revisit it if an effect must
be added by a plugin (from docs/design/0.2.0-requirement-tradeoffs.md, high).

Migrated from `R-OPS-001` in `docs/spec/ops.md`, its first obligation.
