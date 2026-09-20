# Modules

**Crate:** `meowctl-module`
**Design:** `docs/design/0.2.0-rust-rewrite.md` §3
**v0.1.0 equivalent:** `internal/mvs/`, `internal/starlark/loader/`

## Scope

This component resolves the module graph: version selection, fetching from the
registry or from GitHub, integrity verification, the on-disk cache, and local
or remote overrides. It hands `meowctl-starlark` a verified file for a URL and
hands `meowctl-config` the resolution to lock.

## Boundary

Resolution, fetching, caching, verification, and the resolved values written to
`deps.lock`.

## Version selection

**[R-MODULE-001]** Version selection MUST be Minimal Version Selection over the
dependency graph, selecting for each module the maximum version any path
requires. `internal/mvs/mvs.go` is the algorithm.

**[R-MODULE-002]** Versions MUST be compared by semver ordering, with the
literal `none` ordering below every valid version.

**[R-MODULE-003]** A dependency declaring a version that is not valid semver
MUST fail resolution naming the module and the version, rather than being
skipped or treated as `none`.

**[R-MODULE-004]** A resolution MUST be deterministic: the same graph MUST
produce the same build list in the same order on every run and on every
machine.

**[R-MODULE-005]** A cycle in the dependency graph MUST terminate rather than
loop. `v0.1.0` guards this by not re-expanding a module at a version it has
already visited.

## Sources

**[R-MODULE-010]** A registry module MUST be resolved through the registry
index, which lists each module's versions, tarball URLs, and integrity hashes.

**[R-MODULE-011]** A GitHub module MUST be named `github:owner/repo@ref`, where
the ref is a tag or a branch, and MUST be resolved to the commit SHA that ref
pointed to at resolution time. The SHA MUST be recorded so later fetches are
reproducible; see [R-CONFIG-020].

**[R-MODULE-012]** A GitHub module's transitive dependencies MUST be read from
its own manifest and walked, so an aggregate module brings its dependencies
with it.

**[R-MODULE-013]** Fetching MUST use HTTPS and MUST NOT shell out to `git`.
`v0.1.0` fetches a tarball over pure-Go HTTPS, and a machine being bootstrapped
may not have `git` yet.

## Overrides

**[R-MODULE-020]** `replace(name, path)` MUST serve every file for that module
from the local directory, skipping fetch, cache, and integrity verification.
The lock entry MUST record that the module is replaced and where it points; see
[R-CONFIG-020].

**[R-MODULE-021]** `replace(name, source)` MUST resolve the module from the
replacement source, and MUST verify its integrity normally: a remote fork is
not more trusted than the original.

**[R-MODULE-022]** An override in `deps.local.mod` MUST win over the same
module in `deps.mod`, because the local file is how one machine differs from
the rest.

## Integrity

**[R-MODULE-030]** Every fetched tarball MUST be verified against the integrity
hash from the index before it is extracted, and every extracted file MUST be
verified against its recorded per-file hash before it is evaluated.

The per-file hashes are recorded in the cache, beside the extracted files,
rather than in `deps.lock`. `deps.lock` has a `files` table for them and
`v0.1.0` never writes one, so a `v0.2.0` that filled it in would both break the
byte-identical lock [R-CONFIG-023] requires and lose the hashes the first time
a `v0.1.0` binary rewrote the entry, because its writer replaces the whole
entry. The hashes say whether the cache still holds what was extracted, and
that question is answerable where the cache is.

**[R-MODULE-031]** A hash mismatch MUST fail with the module code, MUST name
the module, the expected hash, and the actual one, and MUST NOT leave the
extracted files in the cache.

**[R-MODULE-032]** Extraction MUST preserve the executable bit from the tar
entry, deriving the on-disk mode as `0o644` or `0o755`. `v0.1.0` fixed this in
`fix: correct module updates`, and getting it wrong breaks every component that
executes a file it shipped.

**[R-MODULE-033]** Extraction MUST reject an entry whose path escapes the
module root. A tarball is remote input.

## The cache

**[R-MODULE-040]** A module MUST be cached under the cache directory keyed by
name and by resolved version, or by commit SHA for a GitHub module; see
[R-COMMON-021].

**[R-MODULE-041]** A cached module whose per-file hashes match what was recorded
at extraction MUST be used without a network request.

**[R-MODULE-042]** A cached module whose files do not match the recorded hashes
MUST be re-fetched rather than used, because the cache is not authoritative. A
cache with no record at all MUST be treated the same way: the module was
extracted by something that did not write one, and what it holds is unknown.

**[R-MODULE-043]** Resolution MUST work offline when every module in the lock
is cached and verified. A shell hook that triggers a resolution on a machine
with no network is otherwise a hang.

**[R-MODULE-044]** The record MUST be written inside the cache directory under
a name that cannot be loaded as a module file, and MUST cover the tarball's
integrity as well as the per-file hashes. `v0.1.0` writes half of it already,
as the `.sri` sidecar that carries the tarball hash, and the format here MUST
keep that file readable by `v0.1.0`: the two binaries share one cache.

## Locking

**[R-MODULE-050]** A module already present in the lock MUST be used at the
locked version without running selection again. Selection runs when a module is
absent, or when the caller asked to ignore the lock.

**[R-MODULE-051]** Syncing MUST write the full resolution: version, source,
integrity, and commit SHA where applicable. The per-file hashes go to the cache
record rather than to the lock; [R-MODULE-030] says why.

**[R-MODULE-052]** Syncing `deps.mod` and `deps.local.mod` MUST produce their
own lock files, and a module in both MUST resolve independently in each.

**[R-MODULE-053]** An upgrade MUST clear the locked version for the named
modules and re-resolve them, leaving the rest of the lock untouched.

## Failure paths

**[R-MODULE-060]** A registry index that cannot be fetched MUST fail with the
module code and MUST say whether the failure was the network, the status code,
or the parse.

**[R-MODULE-061]** A module named in `deps.mod` but absent from the index MUST
be reported by name, distinguished from a version of it that does not exist.

**[R-MODULE-062]** A requirement no version satisfies MUST name the module and
the constraint.

**[R-MODULE-063]** A fetch interrupted partway MUST NOT leave a partial tarball
or a partially extracted module that a later run would treat as cached; see
[R-FS-031].

**[R-MODULE-064]** A `replace` pointing at a path that does not exist MUST fail
naming the path, not fall back to fetching the original.

## Parity with v0.1.0

The selection algorithm, the URL forms, the SRI verification, the cache layout,
the executable-bit handling, and the local-overlay precedence all come from
`internal/mvs/mvs.go` and `internal/starlark/loader/`. The lock this component
produces must be byte-identical to `v0.1.0`'s; see [R-CONFIG-023].

Three things are new, and the first is the largest.

`v0.1.0` does not verify a registry module's files at all. It checks the
tarball against the index hash when it downloads one, writes the `.sri`
sidecar, and from then on reads whatever is in the cache directory and
evaluates it. Anything that can write to `~/.cache` can change what a hook
runs, and nothing notices. [R-MODULE-030], [R-MODULE-041], [R-MODULE-042] and
[R-MODULE-044] close that, which is why the parity claim here is about the
formats and the URLs rather than about the checks.

The `.sri` sidecar stays exactly as `v0.1.0` writes it so the two binaries
share a cache; the per-file hashes go in a second file beside it, which
`v0.1.0` ignores. A cache populated by `v0.1.0` therefore has no per-file
record, and [R-MODULE-042] re-fetches it once.

[R-MODULE-043] is also stated rather than reproduced. `v0.1.0` reaches the
network whenever a module is absent from the lock and never says that a fully
locked, fully cached configuration must not, which matters for
`meowctl hook shell`, on every shell spawn.

[R-MODULE-063] is the third. `v0.1.0` extracts straight into the cache
directory, so an interrupted extraction leaves a directory that `os.Stat`
finds and every later run treats as a complete module. Extracting beside it and
renaming makes the directory's existence mean what the code already assumes it
means.
