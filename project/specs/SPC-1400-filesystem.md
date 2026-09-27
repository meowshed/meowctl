---
id: SPC-1400
artifact: spec
status: live
revised: 2026-09-27
checked-at:
states: [REQ-1401, REQ-1402, REQ-1403, REQ-1404, REQ-1405, REQ-1406, REQ-1410, REQ-1411, REQ-1412, REQ-1413, REQ-1414, REQ-1420, REQ-1421, REQ-1422, REQ-1430, REQ-1431, REQ-1432, REQ-1433, REQ-1500, REQ-1501, REQ-1502, REQ-1503, REQ-1504, REQ-1505, REQ-1506, REQ-1507, REQ-1508, REQ-1570]
---

# The filesystem trait and its three implementations

## Scope

This part is the `meowctl-fs` crate, designed in
`docs/design/0.2.0-rust-rewrite.md` §3 against defect #4, and replaces the
direct `os.*` calls throughout `internal/ctx/methods.go` in `v0.1.0` (from
docs/spec/fs.md, high).

This component owns every filesystem effect the rest of the workspace performs,
behind one trait with three implementations. Nothing above it calls `std::fs`
directly (from docs/spec/fs.md, high).

A dry run here is a different implementation, so a forgotten dry-run branch
can't be expressed (from docs/spec/fs.md, high).

## Boundary

The `FileSystem` trait, its three implementations, and the errors they return.
Path validation lives in `meowctl-common`; see [REQ-1022] (from docs/spec/fs.md,
high).

| Surface | What it is |
| --- | --- |
| `FileSystem` | The trait every filesystem effect goes through (from docs/spec/fs.md, high) |
| `RealFs` | The implementation that acts on the real filesystem (from docs/spec/fs.md, high) |
| `DryRunFs` | The implementation that records mutations and reads through to disk (from docs/spec/fs.md, high) |
| `MemFs` | The in-memory implementation for tests (from docs/spec/fs.md, high) |
| Errors | A typed error per failure, carrying the path and the operation (from docs/spec/fs.md, high) |

## Behaviour

### The trait

`FileSystem` covers exactly these operations: reading, writing, appending to,
removing and copying a file; creating and removing a symlink and reading its
target; creating and listing a directory; renaming; and querying metadata,
including whether a path exists and whether it is a symlink [REQ-1401] (from
docs/spec/fs.md, high).

Every method takes an already-resolved absolute path [REQ-1402], and the trait
doesn't expand `~`, join a relative path against a working directory, or consult
the environment [REQ-1500] (from docs/spec/fs.md, high).

A write goes to a temporary file in the destination's directory and is renamed
into place, so a reader never observes a partial file and a crash mid-write
leaves the prior content [REQ-1403] (from docs/spec/fs.md, high).

A created file gets mode `0o600` and a created directory `0o700`, unless the
caller passes an explicit mode [REQ-1404] (from docs/spec/fs.md, high).

The trait exposes setting and reporting a file's executable bit [REQ-1405], and
both are a no-op on a platform without mode bits [REQ-1501] (from
docs/spec/fs.md, high).

`remove` removes a file, a symlink or an empty directory [REQ-1406]. A separate
method removes a directory and everything in it [REQ-1503] (from
docs/spec/fs.md, high).

### Implementations

`RealFs` performs the effect against the real filesystem [REQ-1410] (from
docs/spec/fs.md, high).

`DryRunFs` performs no mutation [REQ-1411]. Every mutating method succeeds,
records the intent, and returns what the caller would have seen [REQ-1504].
Reads pass through to the real filesystem [REQ-1505], except that a read of a
path with a recorded write returns the content of that write [REQ-1412] (from
docs/spec/fs.md, high).

`MemFs` holds the whole tree in memory and touches no disk [REQ-1413] (from
docs/spec/fs.md, high).

The three implementations are interchangeable behind a trait object, and
`meowctl-cli` constructs one from the flags and passes it down [REQ-1414] (from
docs/spec/fs.md, high).

### Symlinks

Creating a symlink over an existing symlink replaces it [REQ-1420] and reports
the prior target to the caller [REQ-1506] (from docs/spec/fs.md, high).

Creating a symlink over an existing regular file with a backup requested renames
the file to a sibling path before the symlink is created [REQ-1507], and reports
that path to the caller [REQ-1508] (from docs/spec/fs.md, high).

On Windows, `RealFs` refuses to create a symlink, with an unsupported-operation
error naming the link path [REQ-1570], because Windows needs a privilege and a
choice between a file link and a directory link, and creating the wrong kind is
worse than refusing (from https://github.com/meowshed/meowctl/pull/57 and
crates/meowctl-fs/src/real.rs:294-308, high).

## Failure paths

| Condition | What happens |
| --- | --- |
| Any method fails | It returns a typed error carrying the path and the operation [REQ-1430] (from docs/spec/fs.md, high) |
| An atomic write fails | The implementation removes its temporary file [REQ-1431] (from docs/spec/fs.md, high) |
| A read names a file that doesn't exist | The error is distinguishable from a read that failed for another reason [REQ-1432] (from docs/spec/fs.md, high) |
| `remove` is given a directory that isn't empty | It refuses [REQ-1502] (from docs/spec/fs.md, high) |
| A symlink is created over a regular file and no backup was requested | The call fails [REQ-1421] (from docs/spec/fs.md, high) |
| A symlink is created on Windows | The call fails with an unsupported-operation error naming the link path [REQ-1570] (from crates/meowctl-fs/src/real.rs:294-308, high) |
| `remove_symlink` names a path that isn't a symlink | The call fails and deletes nothing [REQ-1422] (from docs/spec/fs.md, high) |
| `DryRunFs` sees a reason `RealFs` would fail: a write into a directory that doesn't exist and wasn't recorded as created, or a symlink over a regular file with no backup requested | `DryRunFs` fails as `RealFs` would [REQ-1433] (from docs/spec/fs.md, high) |
