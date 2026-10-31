---
id: TSK-0046
artifact: task
status: approved
revised: 2026-10-31
bug: BUG-0046
closes: []
---

# Follow a symlink when a dry run asks whether a directory exists

`DryRunFs` treats a symlink to a directory as that directory when it checks a
write's parent, and still refuses a write through a link to nothing.

## Acceptance criteria

1. Given a directory reached through an absolute or a relative symlink, when a
   dry run writes a file into it, then the write succeeds. Closed by:
   `a_write_through_a_linked_directory_is_not_refused` in
   `crates/meowctl-fs/tests/dry_run.rs`.
2. Given a symlink to a path that doesn't exist, when a dry run writes through
   it, then the write fails with the missing directory. Closed by:
   `a_write_through_a_dangling_link_is_refused`.

## What to do

In `directory_would_exist` in `crates/meowctl-fs/src/dry_run.rs`, follow an
`Entry::Symlink` to its target, relative to the link's directory when the
target is relative, with a bound on the number of hops, and answer for the
target.

## Depends on

None.

## Left alone

`RealFs::entry` keeps reporting a symlink as a symlink, because the ops that
replace links depend on it.

## Evidence

Not yet.
