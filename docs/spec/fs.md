# Filesystem

**Crate:** `meowctl-fs`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3, defect #4
**v0.1.0 equivalent:** direct `os.*` calls throughout `internal/ctx/methods.go`

## Scope

This component owns every filesystem effect the rest of the workspace performs,
behind one trait with three implementations. Nothing above it calls `std::fs`
directly.

It exists because of defect #4. In `v0.1.0` a dry run is `if c.caps.DryRun`
repeated through every effectful method, which means the guarantee "a dry run
writes nothing" holds only as long as nobody forgets a branch. Somebody did:
`fix(apply): dry-run claimed work that the runner skips`. Here a dry run is a
different implementation, so forgetting is not expressible.

## Boundary

The `FileSystem` trait, its three implementations, and the errors they return.
Path validation lives in `meowctl-common`; see [R-COMMON-022].

## The trait

**[R-FS-001]** `FileSystem` MUST cover exactly the operations the rest of the
workspace needs: reading a file, writing a file, appending to a file, removing
a file, copying a file, creating and removing a symlink, reading a symlink's
target, creating a directory, listing a directory, renaming, and querying
metadata including whether a path exists and whether it is a symlink.

**[R-FS-002]** Every method MUST take an already-resolved absolute path. The
trait MUST NOT expand `~`, join a relative path against a working directory, or
consult the environment, because a dry-run implementation that resolved paths
differently from the real one would report a plan that does not match what runs.

**[R-FS-003]** A write MUST be atomic: the implementation writes a temporary
file in the destination's directory and renames it into place, so a reader
never observes a partial file and a crash mid-write leaves the prior content.
`internal/lock/write.go` and `internal/state/state.go` both do this, and the
lock files depend on it.

**[R-FS-004]** A created file MUST have mode `0o600` and a created directory
`0o700`, matching `v0.1.0`, except where a caller passes an explicit mode.
Configuration holds secrets often enough that the default has to be the narrow
one.

**[R-FS-006]** `remove` MUST remove a file, a symlink, or an empty directory,
and MUST refuse a directory that is not empty. The trait MUST also expose
removing a directory and everything in it, which is a separate method because
the two are different decisions: one is an undo, the other discards a subtree.
The recursive one exists for the module cache, which replaces a module's
directory when it no longer matches what was recorded; see [R-MODULE-042].

`v0.1.0` has no equivalent, because nothing there removes a cache entry: a
module whose files changed under it is evaluated as it stands.

**[R-FS-005]** The trait MUST expose setting a file's executable bit, and
reporting it, and MUST make both a no-op on a platform without mode bits. This
is the one exception [R-FS-004] allows, and it exists because a module tarball
ships scripts meowctl later executes; see [R-MODULE-032]. It is the executable
bit rather than a mode because that is the whole of what the caller decides:
`v0.1.0` normalises every extracted file to `0o644` or `0o755` and nothing
asks for a third value.

## Implementations

**[R-FS-010]** `RealFs` MUST perform the effect against the real filesystem.

**[R-FS-011]** `DryRunFs` MUST perform no mutation. Every mutating method MUST
succeed, record the intent, and return what the caller would have seen. Reads
MUST pass through to the real filesystem, because a hook that reads a file it
just wrote in a dry run has to see plausible content.

**[R-FS-012]** `DryRunFs` MUST answer a read of a path it recorded a write for
with the content of that write, not with what is on disk. Without this a hook
that writes a file and then reads it back takes a different branch under
`--dry-run` than it will under a real run, and the plan stops predicting the
run.

**[R-FS-013]** `MemFs` MUST hold the whole tree in memory and touch no disk. It
exists for tests, and it is what makes the engine testable without a temporary
directory.

**[R-FS-014]** The three implementations MUST be interchangeable behind a trait
object. `meowctl-cli` constructs one from the flags and passes it down; no
other crate chooses.

## Symlinks

**[R-FS-020]** Creating a symlink over an existing symlink MUST replace it, and
the implementation MUST report the prior target so the caller can journal an
inverse. `internal/ctx/methods.go` reads the prior target before replacing, and
[R-OPS-014] depends on getting it.

**[R-FS-021]** Creating a symlink over an existing regular file MUST fail
unless the caller asked for a backup. When a backup is requested, the file MUST
be renamed to a sibling path before the symlink is created, and that path MUST
be reported to the caller.

**[R-FS-022]** Removing a symlink MUST fail when the path is not a symlink, so
that a mistaken `remove_symlink` on a real file cannot delete it.

## Failure paths

**[R-FS-030]** Every method MUST return a typed error carrying the path and the
operation. A caller that surfaces "permission denied" without saying which file
sends the user to read a hook's source to find out.

**[R-FS-031]** A failed atomic write MUST remove its temporary file. `v0.1.0`
does this on every error branch in `internal/lock/write.go`, and leaving
`.meowctl-lock-*.tmp` files behind in the config directory is the failure to
avoid.

**[R-FS-032]** A read of a file that does not exist MUST be distinguishable
from a read that failed for another reason, because callers treat a missing
lock file as a clean slate and a missing config file as an error.

**[R-FS-033]** `DryRunFs` MUST fail where `RealFs` would fail for a reason it
can see: a write into a directory that does not exist and was not recorded as
created, a symlink over a regular file with no backup requested. A dry run that
succeeds where the real run will fail is worse than no dry run.

## Parity with v0.1.0

The mode defaults, the atomic-write discipline, and the symlink replacement
behaviour are reproduced from `internal/ctx/methods.go`, `internal/lock/write.go`
and `internal/state/state.go`.

[R-FS-004] tightens one thing. `v0.1.0` creates a lock file's parent directory
with mode `0o755` in `internal/lock/write.go` and the sentinel's with `0o700`
in `internal/state/state.go`, for the same directory. `0o700` is the value
here, because the config directory holds whatever a user's components put in
it.

[R-FS-012] and [R-FS-033] are new. `v0.1.0`'s dry run is a set of early returns,
so it neither remembers what it pretended to write nor predicts a failure. Both
requirements exist because `--dry-run` is specified here as a prediction of the
run rather than a description of the first step.
